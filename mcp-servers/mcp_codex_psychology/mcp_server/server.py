"""FastMCP server for developer psychology analysis via git patterns."""

from fastmcp import FastMCP
from typing import Optional, List, Dict, Any, Union
import json
from pathlib import Path
from datetime import datetime, timedelta
from git import Repo
import httpx
import ssl
import certifi
import os
from dotenv import load_dotenv

# Load environment variables from .env file
# Look for .env in the parent directory of mcp_server/
env_path = Path(__file__).parent.parent / '.env'
load_dotenv(dotenv_path=env_path)

# Import database functions
from . import database as db

# Kontext API Configuration
KONTEXT_API_KEY = "ktextqxQrFCqCXxjnxsKuUaBvzBIEhEMDMkEHTwkurZUFBXfMuZzaCFMmYgzWrKmaFqxv"
KONTEXT_BASE_URL = "https://staging-api.kontext.dev"
KONTEXT_ORG_ID = "cmh6h4j6c0004pm0k3q9kceit"
KONTEXT_DEVELOPER_ID = "cmh6bce1u000dpe0kae1phdsg"

# Initialize FastMCP server
mcp = FastMCP("codex-psychology")

# Current repository path (in-memory state)
_current_repo_path: Optional[str] = None
_current_repo_id: Optional[int] = None


@mcp.tool()
def set_repository(repo_path: str) -> str:
    """
    Set the git repository to analyze.

    Args:
        repo_path: Absolute path to the git repository

    Returns:
        JSON string with status

    🤖 CODEX: Call this first with the repository path provided by the user.
    """
    global _current_repo_path, _current_repo_id

    repo_path_obj = Path(repo_path)
    if not repo_path_obj.exists():
        return json.dumps({
            "status": "error",
            "message": f"Repository path does not exist: {repo_path}"
        })

    if not (repo_path_obj / ".git").exists():
        return json.dumps({
            "status": "error",
            "message": f"Not a git repository: {repo_path}"
        })

    _current_repo_path = repo_path

    # Get or create repo in database
    _current_repo_id = db.get_or_create_repo(repo_path)

    # Get basic repo info
    try:
        repo = Repo(repo_path)
        branch = repo.active_branch.name
        commit_count = len(list(repo.iter_commits('HEAD', max_count=1000)))

        # Get context from database
        context = db.get_repo_context(repo_path)

        return json.dumps({
            "status": "success",
            "repo_path": repo_path,
            "current_branch": branch,
            "commit_count": min(commit_count, 1000),
            "total_previous_scans": context['repo_info']['total_scans'],
            "message": "Repository set successfully. Ready for analysis."
        })
    except Exception as e:
        return json.dumps({
            "status": "error",
            "message": f"Failed to read repository: {str(e)}"
        })


@mcp.tool()
def analyze_commit_patterns(limit: int = 50) -> str:
    """
    Analyze commit frequency and size patterns.

    Args:
        limit: Number of recent commits to analyze

    Returns:
        JSON string with commit pattern analysis

    🤖 CODEX: Use this to detect:
       - Commitment issues (frequent tiny commits vs rare large commits)
       - Inconsistent commit patterns
       - Rushed/anxious commits (many in short time)
    """
    global _current_repo_path

    if not _current_repo_path:
        return json.dumps({
            "status": "error",
            "message": "No repository set. Call set_repository() first."
        })

    try:
        repo = Repo(_current_repo_path)
        commits = list(repo.iter_commits('HEAD', max_count=limit))

        # Analyze commit sizes and timing
        commit_data = []
        for commit in commits:
            stats = commit.stats.total
            commit_data.append({
                "sha": commit.hexsha[:8],
                "message": commit.message.split('\n')[0],
                "lines_changed": stats['lines'],
                "files_changed": stats['files'],
                "timestamp": commit.committed_datetime.isoformat(),
                "hour_of_day": commit.committed_datetime.hour
            })

        # Calculate patterns
        total_lines = sum(c['lines_changed'] for c in commit_data)
        avg_lines = total_lines / len(commit_data) if commit_data else 0

        # Detect small commits
        small_commits = [c for c in commit_data if c['lines_changed'] < 10]
        large_commits = [c for c in commit_data if c['lines_changed'] > 200]

        # Time distribution
        night_commits = [c for c in commit_data if c['hour_of_day'] >= 22 or c['hour_of_day'] < 6]

        return json.dumps({
            "status": "success",
            "total_commits": len(commit_data),
            "avg_lines_per_commit": round(avg_lines, 1),
            "small_commits_count": len(small_commits),
            "large_commits_count": len(large_commits),
            "night_commits_count": len(night_commits),
            "commits": commit_data[:10],  # First 10 for reference
            "patterns": {
                "has_many_small_commits": len(small_commits) > len(commit_data) * 0.3,
                "has_large_commits": len(large_commits) > 0,
                "commits_at_night": len(night_commits) > len(commit_data) * 0.2,
                "inconsistent_sizing": len(small_commits) > 0 and len(large_commits) > 0
            }
        })
    except Exception as e:
        return json.dumps({
            "status": "error",
            "message": f"Failed to analyze commits: {str(e)}"
        })


@mcp.tool()
def analyze_message_language(limit: int = 50) -> str:
    """
    Analyze commit message language for psychological patterns.

    Args:
        limit: Number of recent commits to analyze

    Returns:
        JSON string with language pattern analysis

    🤖 CODEX: Use this to detect:
       - Minimizing language ("just", "quick", "small")
       - Defensive language ("fix", "oops", "my bad")
       - Perfectionist language ("perfect", "complete", "final")
       - Vague messages that avoid specificity
    """
    global _current_repo_path

    if not _current_repo_path:
        return json.dumps({
            "status": "error",
            "message": "No repository set. Call set_repository() first."
        })

    try:
        repo = Repo(_current_repo_path)
        commits = list(repo.iter_commits('HEAD', max_count=limit))

        # Language patterns to detect
        minimizing_words = ['just', 'quick', 'small', 'minor', 'tiny', 'little']
        defensive_words = ['fix', 'oops', 'my bad', 'sorry', 'mistake', 'bug']
        perfectionist_words = ['perfect', 'complete', 'final', 'done', 'finished']
        vague_words = ['update', 'change', 'stuff', 'things', 'misc']

        minimizing_commits = []
        defensive_commits = []
        perfectionist_commits = []
        vague_commits = []

        for commit in commits:
            msg_lower = commit.message.lower()
            stats = commit.stats.total

            commit_info = {
                "sha": commit.hexsha[:8],
                "message": commit.message.split('\n')[0],
                "lines_changed": stats['lines']
            }

            if any(word in msg_lower for word in minimizing_words):
                minimizing_commits.append(commit_info)
            if any(word in msg_lower for word in defensive_words):
                defensive_commits.append(commit_info)
            if any(word in msg_lower for word in perfectionist_words):
                perfectionist_commits.append(commit_info)
            if any(word in msg_lower for word in vague_words):
                vague_commits.append(commit_info)

        return json.dumps({
            "status": "success",
            "total_commits_analyzed": len(commits),
            "minimizing_commits": {
                "count": len(minimizing_commits),
                "percentage": round(len(minimizing_commits) / len(commits) * 100, 1),
                "examples": minimizing_commits[:5]
            },
            "defensive_commits": {
                "count": len(defensive_commits),
                "percentage": round(len(defensive_commits) / len(commits) * 100, 1),
                "examples": defensive_commits[:5]
            },
            "perfectionist_commits": {
                "count": len(perfectionist_commits),
                "percentage": round(len(perfectionist_commits) / len(commits) * 100, 1),
                "examples": perfectionist_commits[:5]
            },
            "vague_commits": {
                "count": len(vague_commits),
                "percentage": round(len(vague_commits) / len(commits) * 100, 1),
                "examples": vague_commits[:5]
            },
            "patterns": {
                "frequently_minimizes": len(minimizing_commits) > len(commits) * 0.2,
                "frequently_defensive": len(defensive_commits) > len(commits) * 0.15,
                "seeks_perfection": len(perfectionist_commits) > len(commits) * 0.1,
                "often_vague": len(vague_commits) > len(commits) * 0.3
            }
        })
    except Exception as e:
        return json.dumps({
            "status": "error",
            "message": f"Failed to analyze language: {str(e)}"
        })


@mcp.tool()
def compare_message_vs_diff(limit: int = 20) -> str:
    """
    Compare commit messages against actual changes to detect self-deception.

    Args:
        limit: Number of recent commits to analyze

    Returns:
        JSON string with message/diff mismatch analysis

    🤖 CODEX: Use this to detect:
       - Downplaying: "quick fix" but 300 lines changed
       - Overselling: "major refactor" but 5 lines changed
       - Avoidance: vague messages for significant changes
    """
    global _current_repo_path

    if not _current_repo_path:
        return json.dumps({
            "status": "error",
            "message": "No repository set. Call set_repository() first."
        })

    try:
        repo = Repo(_current_repo_path)
        commits = list(repo.iter_commits('HEAD', max_count=limit))

        mismatches = []

        for commit in commits:
            msg = commit.message.split('\n')[0].lower()
            stats = commit.stats.total
            lines = stats['lines']

            # Detect downplaying
            minimizing_words = ['quick', 'small', 'minor', 'tiny', 'little', 'just']
            is_minimizing = any(word in msg for word in minimizing_words)
            is_large_change = lines > 100

            if is_minimizing and is_large_change:
                mismatches.append({
                    "type": "downplaying",
                    "sha": commit.hexsha[:8],
                    "message": commit.message.split('\n')[0],
                    "lines_changed": lines,
                    "observation": f"Message uses minimizing language but changed {lines} lines"
                })

            # Detect vagueness on significant changes
            vague_words = ['update', 'change', 'stuff', 'things', 'misc']
            is_vague = any(word in msg for word in vague_words) and len(msg.split()) < 5

            if is_vague and lines > 50:
                mismatches.append({
                    "type": "vague_on_significant",
                    "sha": commit.hexsha[:8],
                    "message": commit.message.split('\n')[0],
                    "lines_changed": lines,
                    "observation": f"Vague message for {lines} line change"
                })

        return json.dumps({
            "status": "success",
            "total_commits_analyzed": len(commits),
            "mismatches_found": len(mismatches),
            "mismatches": mismatches,
            "patterns": {
                "frequently_downplays": len([m for m in mismatches if m['type'] == 'downplaying']) > 3,
                "avoids_specificity": len([m for m in mismatches if m['type'] == 'vague_on_significant']) > 3
            }
        })
    except Exception as e:
        return json.dumps({
            "status": "error",
            "message": f"Failed to compare messages: {str(e)}"
        })


@mcp.tool()
def get_temporal_patterns(days: int = 30) -> str:
    """
    Analyze when commits happen to detect stress/anxiety patterns.

    Args:
        days: Number of days to look back

    Returns:
        JSON string with temporal pattern analysis

    🤖 CODEX: Use this to detect:
       - Late night commits (anxiety, deadline stress)
       - Weekend work (work-life boundary issues)
       - Burst patterns (procrastination then panic)
    """
    global _current_repo_path

    if not _current_repo_path:
        return json.dumps({
            "status": "error",
            "message": "No repository set. Call set_repository() first."
        })

    try:
        repo = Repo(_current_repo_path)
        since_date = datetime.now() - timedelta(days=days)

        commits = [c for c in repo.iter_commits('HEAD')
                   if c.committed_datetime.replace(tzinfo=None) > since_date]

        if not commits:
            return json.dumps({
                "status": "success",
                "message": f"No commits found in last {days} days"
            })

        # Analyze by time of day
        by_hour = {}
        by_day_of_week = {}

        for commit in commits:
            dt = commit.committed_datetime
            hour = dt.hour
            dow = dt.strftime('%A')

            by_hour[hour] = by_hour.get(hour, 0) + 1
            by_day_of_week[dow] = by_day_of_week.get(dow, 0) + 1

        # Categorize
        night_commits = sum(by_hour.get(h, 0) for h in range(22, 24)) + sum(by_hour.get(h, 0) for h in range(0, 6))
        weekend_commits = by_day_of_week.get('Saturday', 0) + by_day_of_week.get('Sunday', 0)

        return json.dumps({
            "status": "success",
            "period_days": days,
            "total_commits": len(commits),
            "commits_by_hour": by_hour,
            "commits_by_day": by_day_of_week,
            "night_commits": night_commits,
            "weekend_commits": weekend_commits,
            "patterns": {
                "works_late_nights": night_commits > len(commits) * 0.25,
                "works_weekends": weekend_commits > len(commits) * 0.25,
                "consistent_schedule": len(by_hour) < 8  # Commits concentrated in few hours
            }
        })
    except Exception as e:
        return json.dumps({
            "status": "error",
            "message": f"Failed to analyze temporal patterns: {str(e)}"
        })


@mcp.tool()
def get_project_summary() -> str:
    """
    Get overall summary of the repository for context.

    Returns:
        JSON string with project summary

    🤖 CODEX: Call this early to understand the project context.
    """
    global _current_repo_path

    if not _current_repo_path:
        return json.dumps({
            "status": "error",
            "message": "No repository set. Call set_repository() first."
        })

    try:
        repo = Repo(_current_repo_path)

        # Get recent commits
        commits = list(repo.iter_commits('HEAD', max_count=100))

        # Get contributors
        authors = {}
        for commit in commits:
            author = commit.author.name
            authors[author] = authors.get(author, 0) + 1

        # Get file types
        file_types = {}
        try:
            tree = repo.head.commit.tree
            for item in tree.traverse():
                if item.type == 'blob':
                    ext = Path(item.path).suffix
                    if ext:
                        file_types[ext] = file_types.get(ext, 0) + 1
        except:
            pass

        return json.dumps({
            "status": "success",
            "repo_path": _current_repo_path,
            "current_branch": repo.active_branch.name,
            "total_commits_scanned": len(commits),
            "contributors": authors,
            "file_types": file_types,
            "latest_commit": {
                "sha": commits[0].hexsha[:8],
                "message": commits[0].message.split('\n')[0],
                "author": commits[0].author.name,
                "date": commits[0].committed_datetime.isoformat()
            } if commits else None
        })
    except Exception as e:
        return json.dumps({
            "status": "error",
            "message": f"Failed to get project summary: {str(e)}"
        })


@mcp.tool()
def get_repo_context() -> str:
    """
    Get comprehensive repository context with history.

    Returns:
        JSON string with repo context including:
        - Repository metadata
        - Previous scans
        - Flagged issues with occurrence counts
        - Fix attempts history
        - Recurring patterns

    🤖 CODEX: **THE KEY TOOL** - Call this to understand repo history and recurring patterns.
    Like secretsoul's get_session_context, this gives you memory across sessions.
    """
    global _current_repo_path

    if not _current_repo_path:
        return json.dumps({
            "status": "error",
            "message": "No repository set. Call set_repository() first."
        })

    try:
        context = db.get_repo_context(_current_repo_path)
        return json.dumps({
            "status": "success",
            **context
        })
    except Exception as e:
        return json.dumps({
            "status": "error",
            "message": f"Failed to get repo context: {str(e)}"
        })


@mcp.tool()
def flag_repo_issue(issue_type: str, severity: float = 0.5, notes: str = "") -> str:
    """
    Flag a recurring issue pattern (tracks occurrence count).

    Args:
        issue_type: Type of issue (e.g., "minimizing_language", "hardcoded_secret")
        severity: Severity level 0.0-1.0
        notes: Additional context

    Returns:
        JSON string with status

    🤖 CODEX: Use this to track patterns you notice. Each call increments occurrence_count.
    Like secretsoul's flag_theme - helps you remember recurring issues across sessions.
    """
    global _current_repo_id

    if not _current_repo_id:
        return json.dumps({
            "status": "error",
            "message": "No repository set. Call set_repository() first."
        })

    try:
        db.flag_issue(_current_repo_id, issue_type, severity, notes)

        # Get updated context to show occurrence count
        context = db.get_repo_context(_current_repo_path)
        flagged = [f for f in context['flagged_issues'] if f['issue_type'] == issue_type]

        occurrence_count = flagged[0]['occurrence_count'] if flagged else 1

        return json.dumps({
            "status": "success",
            "issue_type": issue_type,
            "occurrence_count": occurrence_count,
            "message": f"Issue flagged. This is occurrence #{occurrence_count} of '{issue_type}'"
        })
    except Exception as e:
        return json.dumps({
            "status": "error",
            "message": f"Failed to flag issue: {str(e)}"
        })


@mcp.tool()
def save_fix_attempt_tool(issue_type: str, fix_description: str, outcome: str = "") -> str:
    """
    Record a fix attempt for an issue.

    Args:
        issue_type: The issue being addressed
        fix_description: What fix was attempted
        outcome: Result of the fix (empty if ongoing)

    Returns:
        JSON string with status

    🤖 CODEX: Track what fixes were tried and their outcomes.
    Helps avoid repeating failed approaches.
    """
    global _current_repo_id

    if not _current_repo_id:
        return json.dumps({
            "status": "error",
            "message": "No repository set. Call set_repository() first."
        })

    try:
        db.save_fix_attempt(_current_repo_id, issue_type, fix_description, outcome)

        return json.dumps({
            "status": "success",
            "message": f"Fix attempt recorded for '{issue_type}'"
        })
    except Exception as e:
        return json.dumps({
            "status": "error",
            "message": f"Failed to save fix attempt: {str(e)}"
        })


# === ROASTING TOOLS ===

@mcp.tool()
def start_session(repo_path: str) -> str:
    """
    Create a new scan session for roasting analysis.

    Args:
        repo_path: Absolute path to the git repository

    Returns:
        JSON string with session_id and discovery questions

    CODEX: Call this to begin a new roasting scan. Returns session ID and
    prompts user with discovery questions about their code quality rating.
    """
    repo_path_obj = Path(repo_path)
    if not repo_path_obj.exists():
        return json.dumps({
            "status": "error",
            "message": f"Repository path does not exist: {repo_path}"
        })

    if not (repo_path_obj / ".git").exists():
        return json.dumps({
            "status": "error",
            "message": f"Not a git repository: {repo_path}"
        })

    try:
        # Check for existing incomplete sessions
        history = db.get_scan_history(repo_path, limit=1)
        if history and history[0].get('completed_at') is None:
            return json.dumps({
                "status": "warning",
                "message": "An incomplete scan session exists. Complete it first or start a new one.",
                "existing_session_id": history[0]['id']
            })

        # Create new session
        session_id = db.create_scan_session(repo_path)

        # Prepare discovery questions
        discovery_questions = [
            "On a scale of 1-10, how would you rate the overall quality of this codebase?",
            "What do you think is the biggest potential of this project?",
            "What concerns do you have about the code, if any?"
        ]

        return json.dumps({
            "status": "success",
            "session_id": session_id,
            "repo_path": repo_path,
            "discovery_questions": discovery_questions,
            "message": "Scan session created. Please answer the discovery questions."
        })
    except Exception as e:
        return json.dumps({
            "status": "error",
            "message": f"Failed to create scan session: {str(e)}"
        })


@mcp.tool()
def close_session(session_id: int) -> str:
    """
    Close/abort an existing scan session without saving results.

    Args:
        session_id: The scan session ID to close

    Returns:
        JSON string with status

    🤖 CODEX: Call this to close incomplete sessions before starting a new analysis.
    Use when you want to clean up without saving results.
    """
    try:
        # Mark session as completed with 0 issues and 0 duration
        db.complete_scan_session(session_id, total_issues=0, duration_ms=0)

        return json.dumps({
            "status": "success",
            "session_id": session_id,
            "message": f"Session {session_id} closed successfully."
        })
    except Exception as e:
        return json.dumps({
            "status": "error",
            "message": f"Failed to close session: {str(e)}"
        })


@mcp.tool()
def submit_discovery_answers(session_id: int, rating: int, potential: str, reasoning: str) -> str:
    """
    Save user's discovery answers for a scan session.

    Args:
        session_id: The scan session ID
        rating: User's quality rating (1-10)
        potential: User's answer about project potential
        reasoning: User's reasoning about concerns

    Returns:
        JSON string with confirmation

    CODEX: Call this after user answers discovery questions. Stores their
    self-assessment for later comparison with actual findings.
    """
    try:
        # Note: Discovery answers are stored in the session metadata
        # For now, we'll track this as a behavioral pattern
        evidence = f"User rated code quality: {rating}/10. Potential: {potential}. Concerns: {reasoning}"

        db.flag_behavioral_pattern(
            session_id=session_id,
            pattern_name="user_self_assessment",
            evidence=evidence,
            severity="info"
        )

        return json.dumps({
            "status": "success",
            "session_id": session_id,
            "message": "Discovery answers saved. Proceed with git and security analysis."
        })
    except Exception as e:
        return json.dumps({
            "status": "error",
            "message": f"Failed to save discovery answers: {str(e)}"
        })


@mcp.tool()
def save_scan_results(session_id: int, git_analysis: str, security_analysis: str) -> str:
    """
    Save complete scan results and mark session as completed.

    Args:
        session_id: The scan session ID
        git_analysis: JSON string of git pattern analysis results
        security_analysis: JSON string of Aikido security scan results

    Returns:
        JSON string with summary

    CODEX: Call this after completing git pattern analysis and Aikido security scan.
    Saves all findings and completes the session.
    """
    try:
        start_time = datetime.now()

        # Parse git analysis
        git_data = json.loads(git_analysis) if isinstance(git_analysis, str) else git_analysis

        # Save git patterns
        if git_data.get('patterns'):
            for pattern_type, detected in git_data['patterns'].items():
                if detected:
                    db.save_git_pattern(
                        session_id=session_id,
                        pattern_type=pattern_type
                    )

        # Save individual commit patterns if available
        if git_data.get('commits'):
            for commit in git_data['commits'][:10]:  # Limit to top 10
                db.save_git_pattern(
                    session_id=session_id,
                    pattern_type="notable_commit",
                    commit_sha=commit.get('sha', ''),
                    commit_message=commit.get('message', ''),
                    lines_changed=commit.get('lines_changed', 0)
                )

        # Parse security analysis
        security_data = json.loads(security_analysis) if isinstance(security_analysis, str) else security_analysis

        # Save security issues
        total_issues = 0
        if security_data.get('issues'):
            for issue in security_data['issues']:
                db.save_security_issue(
                    session_id=session_id,
                    severity=issue.get('severity', 'unknown'),
                    category=issue.get('category', 'uncategorized'),
                    summary=issue.get('summary', ''),
                    issue_url=issue.get('url', '')
                )
                total_issues += 1

                # Track recurring issues
                issue_signature = f"{issue.get('category', 'uncategorized')}:{issue.get('summary', '')[:100]}"
                # Get repo_path from session
                with db.get_db() as conn:
                    session = conn.execute(
                        "SELECT repo_path FROM scan_sessions WHERE id = ?",
                        (session_id,)
                    ).fetchone()
                    if session:
                        db.track_recurring_issue(session['repo_path'], issue_signature)

        # Complete the session
        duration_ms = int((datetime.now() - start_time).total_seconds() * 1000)
        db.complete_scan_session(session_id, total_issues, duration_ms)

        return json.dumps({
            "status": "success",
            "session_id": session_id,
            "total_git_patterns": len(git_data.get('patterns', {})),
            "total_security_issues": total_issues,
            "scan_duration_ms": duration_ms,
            "message": "Scan results saved and session completed."
        })
    except Exception as e:
        return json.dumps({
            "status": "error",
            "message": f"Failed to save scan results: {str(e)}"
        })


@mcp.tool()
def get_scan_history(repo_path: str, limit: int = 10) -> str:
    """
    Get past scan sessions for a repository.

    Args:
        repo_path: Absolute path to the git repository
        limit: Maximum number of scans to return (default: 10)

    Returns:
        Formatted string with scan history for LLM consumption

    CODEX: Use this to see previous roasting sessions for this repo.
    Shows timestamps, issue counts, and completion status.
    """
    try:
        scans = db.get_scan_history(repo_path, limit)

        if not scans:
            return json.dumps({
                "status": "success",
                "message": "No previous scans found for this repository.",
                "scans": []
            })

        # Format for LLM-friendly output
        formatted_scans = []
        for scan in scans:
            formatted_scans.append({
                "session_id": scan['id'],
                "started_at": scan['started_at'],
                "completed_at": scan['completed_at'] or "INCOMPLETE",
                "total_issues": scan['total_issues'],
                "scan_duration_ms": scan['scan_duration_ms']
            })

        return json.dumps({
            "status": "success",
            "repo_path": repo_path,
            "total_scans": len(scans),
            "scans": formatted_scans,
            "message": f"Found {len(scans)} previous scan(s)"
        })
    except Exception as e:
        return json.dumps({
            "status": "error",
            "message": f"Failed to get scan history: {str(e)}"
        })


@mcp.tool()
def query_recurring_issues(repo_path: str, category: Optional[str] = None) -> str:
    """
    Find issues that appear across multiple scans.

    Args:
        repo_path: Absolute path to the git repository
        category: Optional category filter (e.g., "security", "code_smell")

    Returns:
        Formatted string with recurring issues and occurrence counts

    CODEX: Use this to identify patterns that keep appearing across scans.
    These are the issues the developer keeps recreating - prime roasting material.
    """
    try:
        recurring_issues = db.get_recurring_issues_for_repo(repo_path)

        if not recurring_issues:
            return json.dumps({
                "status": "success",
                "message": "No recurring issues found for this repository.",
                "recurring_issues": []
            })

        # Filter by category if specified
        if category:
            recurring_issues = [
                issue for issue in recurring_issues
                if category.lower() in issue['issue_signature'].lower()
            ]

        # Format for LLM
        formatted_issues = []
        for issue in recurring_issues:
            formatted_issues.append({
                "issue_signature": issue['issue_signature'],
                "occurrence_count": issue['occurrence_count'],
                "first_seen": issue['first_seen'],
                "last_seen": issue['last_seen'],
                "persistence": f"Occurred {issue['occurrence_count']} times between {issue['first_seen']} and {issue['last_seen']}"
            })

        return json.dumps({
            "status": "success",
            "repo_path": repo_path,
            "category_filter": category or "all",
            "total_recurring_issues": len(formatted_issues),
            "recurring_issues": formatted_issues,
            "message": f"Found {len(formatted_issues)} recurring issue(s)"
        })
    except Exception as e:
        return json.dumps({
            "status": "error",
            "message": f"Failed to query recurring issues: {str(e)}"
        })


@mcp.tool()
def generate_fix_prompt(issue_id: int) -> str:
    """
    Generate an actionable fix prompt for a security issue.

    Args:
        issue_id: The security issue ID from the database

    Returns:
        Formatted fix prompt for sub-agent execution

    CODEX: Use this to create fix prompts for security issues.
    Returns structured prompt with context, steps, and prevention measures.
    """
    try:
        with db.get_db() as conn:
            # Get the security issue
            issue = conn.execute(
                """SELECT si.*, ss.repo_path, ss.repo_name
                   FROM security_issues si
                   JOIN scan_sessions ss ON si.session_id = ss.id
                   WHERE si.id = ?""",
                (issue_id,)
            ).fetchone()

            if not issue:
                return json.dumps({
                    "status": "error",
                    "message": f"Security issue ID {issue_id} not found"
                })

            # Check if there are previous fix attempts
            fix_attempts = conn.execute(
                """SELECT fix_prompt, success, notes, attempted_at
                   FROM fix_attempts_roasting
                   WHERE issue_id = ?
                   ORDER BY attempted_at DESC
                   LIMIT 3""",
                (issue_id,)
            ).fetchall()

        # Build fix prompt
        fix_prompt = f"""# Security Issue Fix Request

## Issue Details
- **Severity**: {issue['severity']}
- **Category**: {issue['category']}
- **Summary**: {issue['summary']}
- **Repository**: {issue['repo_path']}
- **Issue URL**: {issue['issue_url'] or 'N/A'}

## Your Task
Fix the security issue described above in the repository at `{issue['repo_path']}`.

## Steps
1. Locate the vulnerable code related to: {issue['category']}
2. Understand the security risk: {issue['summary']}
3. Implement a secure fix following best practices
4. Test the fix to ensure it resolves the issue without breaking functionality
5. Document the change in your commit message

## Prevention Measures
After fixing, consider:
- Adding tests to prevent regression
- Checking for similar patterns elsewhere in the codebase
- Updating documentation if security best practices changed
"""

        # Add context from previous attempts if any
        if fix_attempts:
            fix_prompt += "\n## Previous Fix Attempts\n"
            for i, attempt in enumerate(fix_attempts, 1):
                status = "SUCCESSFUL" if attempt['success'] else "FAILED"
                fix_prompt += f"\n### Attempt {i} ({status})\n"
                fix_prompt += f"- **When**: {attempt['attempted_at']}\n"
                fix_prompt += f"- **Notes**: {attempt['notes']}\n"

        return json.dumps({
            "status": "success",
            "issue_id": issue_id,
            "fix_prompt": fix_prompt,
            "severity": issue['severity'],
            "category": issue['category']
        })
    except Exception as e:
        return json.dumps({
            "status": "error",
            "message": f"Failed to generate fix prompt: {str(e)}"
        })


@mcp.tool()
def flag_behavioral_pattern(session_id: int, pattern_name: str, evidence: str, severity: str = "medium") -> str:
    """
    Flag a behavioral pattern detected during analysis.

    Args:
        session_id: The scan session ID
        pattern_name: Name of the pattern (e.g., "minimizing_language", "defensive_commits")
        evidence: Evidence supporting this pattern
        severity: Severity level (low, medium, high)

    Returns:
        JSON string with confirmation and occurrence count

    CODEX: Use this to record behavioral patterns you detect in git analysis.
    Auto-increments occurrence count if pattern was flagged before in this session.
    """
    try:
        # Flag the pattern (this handles increment if exists)
        db.flag_behavioral_pattern(
            session_id=session_id,
            pattern_name=pattern_name,
            evidence=evidence,
            severity=severity
        )

        # Get updated occurrence count
        with db.get_db() as conn:
            result = conn.execute(
                """SELECT occurrence_count FROM behavioral_patterns
                   WHERE session_id = ? AND pattern_name = ?""",
                (session_id, pattern_name)
            ).fetchone()

            occurrence_count = result['occurrence_count'] if result else 1

        return json.dumps({
            "status": "success",
            "session_id": session_id,
            "pattern_name": pattern_name,
            "occurrence_count": occurrence_count,
            "severity": severity,
            "message": f"Behavioral pattern '{pattern_name}' flagged. Occurrence #{occurrence_count} in this session."
        })
    except Exception as e:
        return json.dumps({
            "status": "error",
            "message": f"Failed to flag behavioral pattern: {str(e)}"
        })


@mcp.tool()
async def run_aikido_security_scan(
    repo_path: Optional[str] = None,
    repository_name: Optional[str] = None,
    scan_types: Optional[Union[str, List[str]]] = None
) -> str:
    """
    Run Aikido Security scan on the repository to detect vulnerabilities.

    Integrates with Aikido Security platform to scan for:
    - Code analysis (SAST - Static Application Security Testing)
    - Exposed secrets and credentials
    - Vulnerable dependencies
    - Infrastructure as Code (IaC) misconfigurations

    Args:
        repo_path: Path to repository (defaults to current repo)
        repository_name: Repository name for Aikido (e.g., "graphyn-desktop-gpui")
                        Defaults to basename of repo_path if not provided
        scan_types: List of scan types or comma-separated string
                   Options: code, secrets, dependencies, iac
                   Default: code, secrets, dependencies

    Returns:
        JSON string with scan results
    """
    global _current_repo_path

    try:
        # Use provided repo or fall back to current
        target_repo = repo_path or _current_repo_path
        if not target_repo:
            return json.dumps({
                "status": "error",
                "message": "No repository set. Call set_repository() first or provide repo_path."
            })

        # Normalize scan_types to list
        if isinstance(scan_types, str):
            scan_types_list = [s.strip() for s in scan_types.split(',')]
        elif scan_types:
            scan_types_list = scan_types
        else:
            scan_types_list = ['code', 'secrets', 'dependencies']  # Default

        # Import Aikido integration
        from .aikido_integration import run_aikido_scan

        # Run the scan with scan types
        scan_results = await run_aikido_scan(
            target_repo,
            repository_name=repository_name,
            scan_types=scan_types_list
        )

        # Defensive: check if scan_results indicates error
        if isinstance(scan_results, dict) and scan_results.get("status") == "error":
            return json.dumps({
                "status": "error",
                "error_type": scan_results.get("error_type", "unknown"),
                "message": scan_results.get("message", "Aikido scan failed"),
                "fix": scan_results.get("fix", "Check logs for details")
            })

        # Safe access with default values
        findings_count = scan_results.get("findings_count", 0)
        status_msg = scan_results.get("status", "success")

        return json.dumps({
            "status": status_msg,
            "repo_path": target_repo,
            "scan_types": scan_types_list,
            "scan_results": scan_results,
            "message": f"Aikido security scan completed. Found {findings_count} issues." if findings_count > 0 else "Aikido scan complete. No issues found."
        })

    except KeyError as e:
        return json.dumps({
            "status": "error",
            "error_type": "invalid_response",
            "message": f"Aikido scan returned unexpected format: missing field '{e}'",
            "fix": "Check Aikido scanner output format"
        })
    except Exception as e:
        return json.dumps({
            "status": "error",
            "error_type": "unknown",
            "message": f"Aikido scan failed: {str(e)}",
            "fix": "Check logs and ensure Docker is running with valid API key"
        })


# ====================
# Kontext Integration Tools
# ====================

@mcp.tool()
async def upload_project_to_kontext(
    documentation_path: str = "/Users/resatugurulu/Downloads/kontextd.md"
) -> str:
    """
    Upload the codex-d project documentation to Kontext vault for persistent context storage.

    Args:
        documentation_path: Path to the kontextd.md file

    Returns:
        JSON string with upload status and file ID
    """
    try:
        # Use system cert path for SSL verification (works on macOS)
        cert_path = '/etc/ssl/cert.pem'

        async with httpx.AsyncClient(verify=cert_path) as client:
            # Upload file using correct Kontext API format
            with open(documentation_path, 'rb') as f:
                files = {'file': ('kontextd.md', f, 'text/markdown')}

                upload_response = await client.post(
                    f"{KONTEXT_BASE_URL}/vault/files",
                    headers={
                        'x-api-key': KONTEXT_API_KEY,
                        'x-as-user': KONTEXT_DEVELOPER_ID
                    },
                    files=files,
                    data={
                        'metadata': json.dumps({
                            'project': 'codex-d',
                            'type': 'technical-documentation',
                            'version': '1.0.0',
                            'org_id': KONTEXT_ORG_ID,
                            'developer_id': KONTEXT_DEVELOPER_ID,
                            'components': [
                                'gpui-framework',
                                'mcp-psychology-server',
                                'codex-acp-integration',
                                'sqlite-database',
                                'git-analysis'
                            ]
                        })
                    }
                )

            result = upload_response.json()
            file_id = result.get('id') or result.get('fileId')

        return json.dumps({
            "status": "success",
            "file_id": file_id,
            "message": f"Documentation uploaded successfully to Kontext. File ID: {file_id}",
            "result": result
        })

    except Exception as e:
        return json.dumps({
            "status": "error",
            "message": f"Failed to upload to Kontext: {str(e)}"
        })


@mcp.tool()
async def query_codex_context(
    query: str,
    top_k: int = 5
) -> str:
    """
    Query the Kontext vault for codex-d project context and documentation.

    Args:
        query: Question about the codex-d project
        top_k: Number of context chunks to retrieve

    Returns:
        JSON string with search results
    """
    try:
        # Use system cert path for SSL verification (works on macOS)
        cert_path = '/etc/ssl/cert.pem'

        async with httpx.AsyncClient(verify=cert_path) as client:
            # Query vault using correct Kontext API format
            query_response = await client.get(
                f"{KONTEXT_BASE_URL}/vault/files",
                headers={
                    'x-api-key': KONTEXT_API_KEY,
                    'x-as-user': KONTEXT_DEVELOPER_ID
                },
                params={
                    'search': query,
                    'limit': top_k
                }
            )

            result = query_response.json()

        return json.dumps({
            "status": "success",
            "query": query,
            "results": result,
            "metadata": {
                "source": "kontext-vault",
                "results_count": len(result) if isinstance(result, list) else 0
            }
        })

    except Exception as e:
        return json.dumps({
            "status": "error",
            "message": f"Failed to query Kontext: {str(e)}"
        })


@mcp.tool()
async def get_codex_system_prompt() -> str:
    """
    Get a comprehensive system prompt with all codex-d project context.

    Returns:
        JSON string with structured system prompt including project details
    """
    try:
        # Construct enhanced system prompt with Kontext integration
        system_prompt = """
You are Codex, a code psychology analyst with deep knowledge of the codex-d project.

## Project Context:
Full project documentation is stored in Kontext vault and can be queried using query_codex_context().

## Core Architecture:
- GPUI Framework (Rust) for UI
- MCP Psychology Server (Python) for analysis
- SQLite database for pattern storage
- 22 specialized MCP tools for behavioral analysis (18 + 4 Kontext tools)

## Your Capabilities:
1. Analyze developer psychology through commit patterns
2. Detect recurring behavioral issues
3. Provide actionable fix recommendations
4. Track remediation attempts
5. Integrate security scanning via Aikido
6. Store and retrieve project context from Kontext vault

## Mandatory Tool Sequence:
Always execute these tools first:
1. start_session(repo_path)
2. set_repository(repo_path)
3. analyze_commit_patterns(limit=50)
4. analyze_message_language()
5. compare_message_vs_diff()
6. get_temporal_patterns()
7. run_aikido_security_scan(repo_path)
8. DEEPWIKI_BOH8VT8Z__ASK_QUESTION(question)

## Kontext Integration:
- upload_project_to_kontext() - Upload documentation
- query_codex_context(query) - Search stored documentation
- save_analysis_to_kontext(session_id, summary, repo_path) - Persist analysis results
"""

        return json.dumps({
            "status": "success",
            "system_prompt": system_prompt,
            "kontext_available": True,
            "org_id": KONTEXT_ORG_ID,
            "developer_id": KONTEXT_DEVELOPER_ID
        })

    except Exception as e:
        return json.dumps({
            "status": "error",
            "message": f"Failed to get system prompt: {str(e)}"
        })


@mcp.tool()
async def save_analysis_to_kontext(
    session_id: int,
    analysis_summary: str,
    repo_path: str
) -> str:
    """
    Save the analysis results to Kontext for future reference and learning.

    Args:
        session_id: Current analysis session ID
        analysis_summary: Complete analysis findings (JSON string)
        repo_path: Repository that was analyzed

    Returns:
        JSON string with confirmation message and file ID
    """
    try:
        # Parse analysis summary
        analysis_results = json.loads(analysis_summary) if isinstance(analysis_summary, str) else analysis_summary

        # Prepare analysis document
        analysis_doc = f"""# Analysis Session: {session_id}
**Date**: {datetime.now().isoformat()}
**Repository**: {repo_path}

## Findings Summary
{json.dumps(analysis_results.get('summary', {}), indent=2)}

## Critical Issues
{json.dumps(analysis_results.get('critical', []), indent=2)}

## Behavioral Patterns
{json.dumps(analysis_results.get('patterns', []), indent=2)}

## Security Scan Results
{json.dumps(analysis_results.get('security', {}), indent=2)}

## Recommendations
{json.dumps(analysis_results.get('recommendations', []), indent=2)}

## Metadata
- Tool calls executed: {analysis_results.get('tool_count', 0)}
- Processing time: {analysis_results.get('duration', 'N/A')}
- Severity distribution: {json.dumps(analysis_results.get('severity_counts', {}), indent=2)}
"""

        # Save to temp file
        temp_path = f"/tmp/analysis_{session_id}.md"
        with open(temp_path, 'w') as f:
            f.write(analysis_doc)

        # Use system cert path for SSL verification (works on macOS)
        cert_path = '/etc/ssl/cert.pem'

        # Upload to Kontext using correct API format
        async with httpx.AsyncClient(verify=cert_path) as client:
            with open(temp_path, 'rb') as f:
                files = {'file': (f'analysis_{session_id}.md', f, 'text/markdown')}

                upload_response = await client.post(
                    f"{KONTEXT_BASE_URL}/vault/files",
                    headers={
                        'x-api-key': KONTEXT_API_KEY,
                        'x-as-user': KONTEXT_DEVELOPER_ID
                    },
                    files=files,
                    data={
                        'metadata': json.dumps({
                            'type': 'analysis-results',
                            'session_id': session_id,
                            'repo': repo_path,
                            'org_id': KONTEXT_ORG_ID,
                            'timestamp': datetime.now().isoformat()
                        })
                    }
                )

            result = upload_response.json()
            file_id = result.get('id') or result.get('fileId')

        # Clean up temp file
        os.unlink(temp_path)

        return json.dumps({
            "status": "success",
            "file_id": file_id,
            "message": f"Analysis saved to Kontext. File ID: {file_id}",
            "result": result
        })

    except Exception as e:
        return json.dumps({
            "status": "error",
            "message": f"Failed to save analysis to Kontext: {str(e)}"
        })


if __name__ == "__main__":
    # Run the MCP server
    mcp.run()
