"""
Aikido Security Integration for Code Analysis

Integrates with Aikido Security platform via Docker local scanner to:
1. Run security scans on repositories
2. Detect vulnerabilities (SAST, secrets, dependencies, IaC)
3. Generate comprehensive security reports

Aikido Local Scanner: https://www.aikido.dev/docs/local-scanner
"""

import os
import subprocess
import platform
import shutil
from typing import Dict, List, Optional
from dataclasses import dataclass
import json
import asyncio


@dataclass
class SecurityFinding:
    """Security vulnerability finding from Aikido"""
    severity: str  # critical, high, medium, low
    title: str
    description: str
    file_path: str
    line_number: Optional[int]
    cwe_id: Optional[str]
    remediation: str


class DockerNotAvailableError(Exception):
    """Raised when Docker is not available"""
    pass


class AikidoAPIKeyMissingError(Exception):
    """Raised when AIKIDO_API_KEY is not configured"""
    pass


class AikidoScanner:
    """Interface to Aikido Security scanning via Docker"""

    SCANNER_IMAGE = "aikidosecurity/local-scanner:latest"
    SCANNER_VERSION = "1.0.109"

    def __init__(self, api_key: Optional[str] = None):
        """
        Initialize Aikido scanner

        Args:
            api_key: Aikido API key (defaults to AIKIDO_API_KEY env var)

        Raises:
            DockerNotAvailableError: If Docker is not installed or daemon not running
            AikidoAPIKeyMissingError: If API key not provided
        """
        self.api_key = api_key or os.getenv("AIKIDO_API_KEY")
        if not self.api_key:
            raise AikidoAPIKeyMissingError(
                "AIKIDO_API_KEY not set. Please configure in .env file or environment."
            )

        # Verify Docker availability
        self.verify_docker()

        # Detect platform for Docker compatibility
        self.platform_flag = self._get_platform_flag()

    def verify_docker(self) -> bool:
        """
        Verify Docker is installed and daemon is running.

        Returns:
            True if Docker is available

        Raises:
            DockerNotAvailableError: If Docker is not available
        """
        # Check Docker CLI exists
        if not shutil.which('docker'):
            raise DockerNotAvailableError(
                "Docker CLI not found. Please install Docker Desktop from https://docker.com"
            )

        # Check Docker daemon is running
        try:
            result = subprocess.run(
                ['docker', 'ps'],
                capture_output=True,
                timeout=5,
                text=True
            )
            if result.returncode != 0:
                raise DockerNotAvailableError(
                    f"Docker daemon is not running. Error: {result.stderr}"
                )
            return True

        except subprocess.TimeoutExpired:
            raise DockerNotAvailableError(
                "Docker daemon not responding. Please start Docker Desktop."
            )
        except Exception as e:
            raise DockerNotAvailableError(f"Docker verification failed: {str(e)}")

    def _get_platform_flag(self) -> List[str]:
        """
        Determine platform flag for Docker compatibility.

        Returns ARM64 Macs need --platform linux/amd64 for Aikido scanner.

        Returns:
            List of Docker platform arguments
        """
        machine = platform.machine().lower()
        if machine in ['arm64', 'aarch64']:
            # ARM64 Mac - need platform emulation
            return ['--platform', 'linux/amd64']
        return []

    async def scan_repository(
        self,
        repo_path: str,
        repository_name: Optional[str] = None,
        scan_types: Optional[List[str]] = None
    ) -> Dict:
        """
        Run Aikido security scan on repository using Docker.

        Args:
            repo_path: Absolute path to repository to scan
            repository_name: Repository name for Aikido (e.g., "graphyn-desktop-gpui")
            scan_types: Types of scans to run. Options:
                       - 'code' (SAST)
                       - 'secrets' (exposed credentials)
                       - 'dependencies' (vulnerable packages)
                       - 'iac' (infrastructure as code)

        Returns:
            Scan results with findings

        Raises:
            ValueError: If repo_path doesn't exist
            subprocess.TimeoutExpired: If scan exceeds timeout
        """
        if not scan_types:
            scan_types = ["secrets", "dependencies", "code"]

        # Validate repository path
        repo_path = os.path.abspath(repo_path)
        if not os.path.exists(repo_path):
            raise ValueError(f"Repository path does not exist: {repo_path}")

        if not os.path.isdir(repo_path):
            raise ValueError(f"Repository path is not a directory: {repo_path}")

        # Extract repository name from path if not provided
        if not repository_name:
            repository_name = os.path.basename(repo_path)

        # Auto-detect git branch
        try:
            branch_result = subprocess.run(
                ['git', 'rev-parse', '--abbrev-ref', 'HEAD'],
                cwd=repo_path,
                capture_output=True,
                text=True,
                timeout=5
            )
            branch_name = branch_result.stdout.strip() if branch_result.returncode == 0 else 'main'
        except Exception:
            branch_name = 'main'  # Fallback to main

        print(f"[Aikido] Starting scan of {repo_path}")
        print(f"[Aikido] Repository name: {repository_name}")
        print(f"[Aikido] Branch name: {branch_name}")
        print(f"[Aikido] Scan types: {', '.join(scan_types)}")

        # Build Docker command
        cmd = [
            'docker', 'run', '--rm',
            *self.platform_flag,
            '-v', f'{repo_path}:/code',
            '-e', f'AIKIDO_API_KEY={self.api_key}',
            self.SCANNER_IMAGE,
            'scan', '/code',
            '--repositoryname', repository_name,
            '--branchname', branch_name,
            '--scan-types', *scan_types,
            '--debug'
        ]

        # Execute Docker scanner
        try:
            process = subprocess.Popen(
                cmd,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True
            )

            # Stream output for progress tracking
            findings_count = 0
            output_lines = []

            for line in process.stdout:
                line = line.strip()
                if line:
                    print(f"[Aikido] {line}")
                    output_lines.append(line)

                    # Count findings in output
                    if 'found' in line.lower() and 'issue' in line.lower():
                        try:
                            # Try to extract finding count from output
                            import re
                            match = re.search(r'(\d+)\s+issue', line)
                            if match:
                                findings_count = int(match.group(1))
                        except:
                            pass

            # Wait for completion (15 minute timeout)
            try:
                returncode = process.wait(timeout=900)
            except subprocess.TimeoutExpired:
                process.kill()
                raise subprocess.TimeoutExpired(
                    cmd, 900,
                    "Scan exceeded 15 minute timeout. Try scanning a smaller directory."
                )

            # Check for errors
            if returncode != 0:
                stderr = process.stderr.read()
                raise RuntimeError(
                    f"Aikido scanner failed with exit code {returncode}.\n"
                    f"Error: {stderr}\n"
                    f"This usually means:\n"
                    f"  - API key is invalid\n"
                    f"  - Network connection failed\n"
                    f"  - Repository path is inaccessible"
                )

            # Parse results from output
            results = self._parse_scan_output(output_lines, findings_count)

            print(f"[Aikido] Scan complete. Found {results['findings_count']} issues.")

            return results

        except Exception as e:
            print(f"[Aikido] Scan error: {str(e)}")
            raise

    def _parse_scan_output(self, output_lines: List[str], findings_count: int) -> Dict:
        """
        Parse Aikido scanner console output into structured results.

        Note: Aikido scanner sends detailed results to cloud platform.
        This parses the console output for summary information.

        Args:
            output_lines: Console output from scanner
            findings_count: Number of findings detected

        Returns:
            Structured scan results
        """
        # Extract severity counts from output
        severity_counts = {
            "critical": 0,
            "high": 0,
            "medium": 0,
            "low": 0
        }

        findings = []

        for line in output_lines:
            line_lower = line.lower()

            # Try to extract severity information
            if 'critical' in line_lower:
                try:
                    import re
                    match = re.search(r'(\d+)\s+critical', line_lower)
                    if match:
                        severity_counts['critical'] = int(match.group(1))
                except:
                    pass

            if 'high' in line_lower:
                try:
                    import re
                    match = re.search(r'(\d+)\s+high', line_lower)
                    if match:
                        severity_counts['high'] = int(match.group(1))
                except:
                    pass

            if 'medium' in line_lower:
                try:
                    import re
                    match = re.search(r'(\d+)\s+medium', line_lower)
                    if match:
                        severity_counts['medium'] = int(match.group(1))
                except:
                    pass

            if 'low' in line_lower:
                try:
                    import re
                    match = re.search(r'(\d+)\s+low', line_lower)
                    if match:
                        severity_counts['low'] = int(match.group(1))
                except:
                    pass

            # Extract individual findings if present in output
            if '│' in line and any(sev in line_lower for sev in ['critical', 'high', 'medium', 'low']):
                # This looks like a finding row
                findings.append({
                    "severity": "unknown",
                    "title": line.strip(),
                    "description": "See Aikido platform for details",
                    "file_path": "unknown",
                    "line_number": None,
                    "cwe_id": None,
                    "remediation": "Check Aikido platform for remediation steps"
                })

        # If no detailed findings found, create summary entry
        if not findings and findings_count > 0:
            findings.append({
                "severity": "info",
                "title": f"{findings_count} security issues detected",
                "description": "Detailed findings are available in your Aikido dashboard",
                "file_path": "See Aikido platform",
                "line_number": None,
                "cwe_id": None,
                "remediation": "Login to app.aikido.dev to view detailed findings and remediation steps"
            })

        return {
            "status": "success",
            "findings_count": findings_count,
            "severity_counts": severity_counts,
            "findings": findings,
            "notes": [
                "Scan completed successfully",
                "Detailed results are available in your Aikido dashboard at https://app.aikido.dev",
                "Use the Aikido platform to view full vulnerability details, CVE information, and remediation guidance"
            ]
        }

    def _finding_to_dict(self, finding: SecurityFinding) -> Dict:
        """Convert SecurityFinding to dictionary"""
        return {
            "severity": finding.severity,
            "title": finding.title,
            "description": finding.description,
            "file_path": finding.file_path,
            "line_number": finding.line_number,
            "cwe_id": finding.cwe_id,
            "remediation": finding.remediation,
        }


async def run_aikido_scan(
    repo_path: str,
    repository_name: Optional[str] = None,
    scan_types: Optional[List[str]] = None
) -> Dict:
    """
    Convenience function to run Aikido security scan.

    Args:
        repo_path: Path to repository
        repository_name: Repository name for Aikido (defaults to basename of repo_path)
        scan_types: Optional list of scan types to run
                   Options: 'code', 'secrets', 'dependencies', 'iac'

    Returns:
        Scan results

    Raises:
        DockerNotAvailableError: If Docker is not available
        AikidoAPIKeyMissingError: If API key not configured
        ValueError: If repo_path is invalid
    """
    try:
        scanner = AikidoScanner()
        return await scanner.scan_repository(repo_path, repository_name=repository_name, scan_types=scan_types)
    except DockerNotAvailableError as e:
        return {
            "status": "error",
            "error_type": "docker_unavailable",
            "message": str(e),
            "fix": "Install Docker Desktop from https://docker.com and ensure it's running"
        }
    except AikidoAPIKeyMissingError as e:
        return {
            "status": "error",
            "error_type": "api_key_missing",
            "message": str(e),
            "fix": "Set AIKIDO_API_KEY in .env file or environment variable"
        }
    except ValueError as e:
        return {
            "status": "error",
            "error_type": "invalid_path",
            "message": str(e),
            "fix": "Provide a valid repository path"
        }
    except subprocess.TimeoutExpired as e:
        return {
            "status": "error",
            "error_type": "timeout",
            "message": str(e),
            "fix": "Try scanning a smaller directory or increase timeout"
        }
    except Exception as e:
        return {
            "status": "error",
            "error_type": "unknown",
            "message": f"Scan failed: {str(e)}",
            "fix": "Check logs for details or contact support"
        }
