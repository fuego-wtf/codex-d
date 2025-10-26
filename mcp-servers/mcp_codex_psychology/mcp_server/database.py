"""SQLite database for persistent repo context tracking."""

import sqlite3
import json
from pathlib import Path
from datetime import datetime
from typing import Optional, Dict, Any, List
from contextlib import contextmanager

# Database path
DB_PATH = Path(__file__).parent / "codex_psychology.db"


@contextmanager
def get_db():
    """Context manager for database connections."""
    conn = sqlite3.connect(str(DB_PATH))
    conn.row_factory = sqlite3.Row
    try:
        yield conn
        conn.commit()
    except Exception:
        conn.rollback()
        raise
    finally:
        conn.close()


def initialize_database():
    """Create all tables if they don't exist."""
    with get_db() as conn:
        # Repositories table
        conn.execute("""
            CREATE TABLE IF NOT EXISTS repositories (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                repo_path TEXT UNIQUE NOT NULL,
                repo_name TEXT NOT NULL,
                first_analyzed_at TEXT NOT NULL,
                last_analyzed_at TEXT NOT NULL,
                total_scans INTEGER DEFAULT 0
            )
        """)

        # Scans table (each enrichment run)
        conn.execute("""
            CREATE TABLE IF NOT EXISTS scans (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                repo_id INTEGER NOT NULL,
                scan_timestamp TEXT NOT NULL,
                git_patterns_json TEXT NOT NULL,
                total_commits_analyzed INTEGER NOT NULL,
                severity REAL NOT NULL,
                scan_duration_ms INTEGER,
                FOREIGN KEY (repo_id) REFERENCES repositories (id)
            )
        """)

        # Issue flags table (recurring patterns with occurrence tracking)
        conn.execute("""
            CREATE TABLE IF NOT EXISTS issue_flags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                repo_id INTEGER NOT NULL,
                issue_type TEXT NOT NULL,
                first_detected_at TEXT NOT NULL,
                last_detected_at TEXT NOT NULL,
                occurrence_count INTEGER DEFAULT 1,
                severity REAL,
                notes TEXT,
                FOREIGN KEY (repo_id) REFERENCES repositories (id),
                UNIQUE(repo_id, issue_type)
            )
        """)

        # Fix attempts table (track what was tried)
        conn.execute("""
            CREATE TABLE IF NOT EXISTS fix_attempts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                repo_id INTEGER NOT NULL,
                issue_type TEXT NOT NULL,
                fix_description TEXT NOT NULL,
                outcome TEXT,
                attempted_at TEXT NOT NULL,
                FOREIGN KEY (repo_id) REFERENCES repositories (id)
            )
        """)

        # Repo profile table (metadata per repo)
        conn.execute("""
            CREATE TABLE IF NOT EXISTS repo_profiles (
                repo_id INTEGER PRIMARY KEY,
                tech_stack TEXT,
                team_size INTEGER,
                project_type TEXT,
                metadata_json TEXT,
                FOREIGN KEY (repo_id) REFERENCES repositories (id)
            )
        """)

        # === ROASTING TABLES ===
        # Scan sessions (one per repo analysis)
        conn.execute("""
            CREATE TABLE IF NOT EXISTS scan_sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                repo_path TEXT NOT NULL,
                repo_name TEXT NOT NULL,
                started_at TEXT,
                completed_at TEXT,
                total_issues INTEGER,
                scan_duration_ms INTEGER
            )
        """)

        # Security issues (from Aikido)
        conn.execute("""
            CREATE TABLE IF NOT EXISTS security_issues (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id INTEGER,
                severity TEXT NOT NULL,
                category TEXT NOT NULL,
                summary TEXT NOT NULL,
                issue_url TEXT,
                fixed_at TEXT,
                fix_notes TEXT,
                FOREIGN KEY (session_id) REFERENCES scan_sessions (id)
            )
        """)

        # Git patterns (behavioral analysis)
        conn.execute("""
            CREATE TABLE IF NOT EXISTS git_patterns (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id INTEGER,
                pattern_type TEXT NOT NULL,
                commit_sha TEXT,
                commit_message TEXT,
                lines_changed INTEGER,
                FOREIGN KEY (session_id) REFERENCES scan_sessions (id)
            )
        """)

        # Behavioral patterns (flagged by Codex)
        conn.execute("""
            CREATE TABLE IF NOT EXISTS behavioral_patterns (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id INTEGER,
                pattern_name TEXT NOT NULL,
                evidence TEXT,
                severity TEXT,
                first_flagged TEXT,
                last_updated TEXT,
                occurrence_count INTEGER DEFAULT 1,
                FOREIGN KEY (session_id) REFERENCES scan_sessions (id)
            )
        """)

        # Recurring issues (cross-scan patterns)
        conn.execute("""
            CREATE TABLE IF NOT EXISTS recurring_issues (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                repo_path TEXT NOT NULL,
                issue_signature TEXT NOT NULL,
                first_seen TEXT,
                last_seen TEXT,
                occurrence_count INTEGER DEFAULT 1,
                UNIQUE(repo_path, issue_signature)
            )
        """)

        # Fix attempts (track what users tried)
        conn.execute("""
            CREATE TABLE IF NOT EXISTS fix_attempts_roasting (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                issue_id INTEGER,
                fix_prompt TEXT,
                attempted_at TEXT,
                success INTEGER,
                notes TEXT,
                FOREIGN KEY (issue_id) REFERENCES security_issues (id)
            )
        """)


def get_or_create_repo(repo_path: str) -> int:
    """Get repo ID or create if doesn't exist."""
    repo_name = Path(repo_path).name
    now = datetime.now().isoformat()

    with get_db() as conn:
        # Try to get existing
        result = conn.execute(
            "SELECT id FROM repositories WHERE repo_path = ?",
            (repo_path,)
        ).fetchone()

        if result:
            # Update last analyzed
            conn.execute(
                "UPDATE repositories SET last_analyzed_at = ? WHERE id = ?",
                (now, result['id'])
            )
            return result['id']

        # Create new
        cursor = conn.execute(
            """INSERT INTO repositories (repo_path, repo_name, first_analyzed_at, last_analyzed_at, total_scans)
               VALUES (?, ?, ?, ?, 0)""",
            (repo_path, repo_name, now, now)
        )
        return cursor.lastrowid


def save_scan(repo_id: int, git_patterns: List[Dict], total_commits: int, severity: float, duration_ms: int = 0):
    """Save scan results."""
    now = datetime.now().isoformat()

    with get_db() as conn:
        conn.execute(
            """INSERT INTO scans (repo_id, scan_timestamp, git_patterns_json, total_commits_analyzed, severity, scan_duration_ms)
               VALUES (?, ?, ?, ?, ?, ?)""",
            (repo_id, now, json.dumps(git_patterns), total_commits, severity, duration_ms)
        )

        # Increment scan count
        conn.execute(
            "UPDATE repositories SET total_scans = total_scans + 1 WHERE id = ?",
            (repo_id,)
        )


def flag_issue(repo_id: int, issue_type: str, severity: float = 0.5, notes: str = ""):
    """Flag an issue - increment occurrence_count if exists, create if new."""
    now = datetime.now().isoformat()

    with get_db() as conn:
        # Try to get existing
        result = conn.execute(
            "SELECT occurrence_count FROM issue_flags WHERE repo_id = ? AND issue_type = ?",
            (repo_id, issue_type)
        ).fetchone()

        if result:
            # Increment occurrence count
            conn.execute(
                """UPDATE issue_flags
                   SET occurrence_count = occurrence_count + 1,
                       last_detected_at = ?,
                       severity = ?,
                       notes = ?
                   WHERE repo_id = ? AND issue_type = ?""",
                (now, severity, notes, repo_id, issue_type)
            )
        else:
            # Create new
            conn.execute(
                """INSERT INTO issue_flags (repo_id, issue_type, first_detected_at, last_detected_at, occurrence_count, severity, notes)
                   VALUES (?, ?, ?, ?, 1, ?, ?)""",
                (repo_id, issue_type, now, now, severity, notes)
            )


def get_repo_context(repo_path: str) -> Dict[str, Any]:
    """
    THE KEY FUNCTION - Get comprehensive repo context (like secretsoul's get_session_context).

    Returns rich context including:
    - Repo metadata
    - Recent scans
    - Flagged issues with occurrence counts
    - Fix attempts history
    - Recurring patterns
    """
    repo_id = get_or_create_repo(repo_path)

    with get_db() as conn:
        # Get repo metadata
        repo = conn.execute(
            "SELECT * FROM repositories WHERE id = ?",
            (repo_id,)
        ).fetchone()

        # Get recent scans (last 5)
        recent_scans = conn.execute(
            """SELECT scan_timestamp, total_commits_analyzed, severity
               FROM scans
               WHERE repo_id = ?
               ORDER BY scan_timestamp DESC
               LIMIT 5""",
            (repo_id,)
        ).fetchall()

        # Get all flagged issues with occurrence counts
        flagged_issues = conn.execute(
            """SELECT issue_type, occurrence_count, severity, first_detected_at, last_detected_at, notes
               FROM issue_flags
               WHERE repo_id = ?
               ORDER BY occurrence_count DESC""",
            (repo_id,)
        ).fetchall()

        # Get recent fix attempts (last 10)
        fix_attempts = conn.execute(
            """SELECT issue_type, fix_description, outcome, attempted_at
               FROM fix_attempts
               WHERE repo_id = ?
               ORDER BY attempted_at DESC
               LIMIT 10""",
            (repo_id,)
        ).fetchall()

        # Get repo profile
        profile = conn.execute(
            "SELECT * FROM repo_profiles WHERE repo_id = ?",
            (repo_id,)
        ).fetchone()

        return {
            "repo_info": {
                "repo_path": repo['repo_path'],
                "repo_name": repo['repo_name'],
                "first_analyzed": repo['first_analyzed_at'],
                "last_analyzed": repo['last_analyzed_at'],
                "total_scans": repo['total_scans']
            },
            "recent_scans": [dict(scan) for scan in recent_scans],
            "flagged_issues": [dict(issue) for issue in flagged_issues],
            "fix_attempts": [dict(attempt) for attempt in fix_attempts],
            "profile": dict(profile) if profile else None
        }


def save_fix_attempt(repo_id: int, issue_type: str, fix_description: str, outcome: str = ""):
    """Save a fix attempt."""
    now = datetime.now().isoformat()

    with get_db() as conn:
        conn.execute(
            """INSERT INTO fix_attempts (repo_id, issue_type, fix_description, outcome, attempted_at)
               VALUES (?, ?, ?, ?, ?)""",
            (repo_id, issue_type, fix_description, outcome, now)
        )


def update_repo_profile(repo_id: int, tech_stack: str = "", team_size: int = 0, project_type: str = "", metadata: Dict = None):
    """Update or create repo profile."""
    with get_db() as conn:
        conn.execute(
            """INSERT OR REPLACE INTO repo_profiles (repo_id, tech_stack, team_size, project_type, metadata_json)
               VALUES (?, ?, ?, ?, ?)""",
            (repo_id, tech_stack, team_size, project_type, json.dumps(metadata or {}))
        )


# === ROASTING HELPER METHODS ===

def create_scan_session(repo_path: str) -> int:
    """Create a new scan session and return session ID."""
    repo_name = Path(repo_path).name
    now = datetime.now().isoformat()

    with get_db() as conn:
        cursor = conn.execute(
            """INSERT INTO scan_sessions (repo_path, repo_name, started_at, total_issues)
               VALUES (?, ?, ?, 0)""",
            (repo_path, repo_name, now)
        )
        return cursor.lastrowid


def complete_scan_session(session_id: int, total_issues: int, duration_ms: int):
    """Mark scan session as completed."""
    now = datetime.now().isoformat()

    with get_db() as conn:
        conn.execute(
            """UPDATE scan_sessions
               SET completed_at = ?, total_issues = ?, scan_duration_ms = ?
               WHERE id = ?""",
            (now, total_issues, duration_ms, session_id)
        )


def save_security_issue(session_id: int, severity: str, category: str, summary: str, issue_url: str = ""):
    """Save a security issue from Aikido scan."""
    with get_db() as conn:
        cursor = conn.execute(
            """INSERT INTO security_issues (session_id, severity, category, summary, issue_url)
               VALUES (?, ?, ?, ?, ?)""",
            (session_id, severity, category, summary, issue_url)
        )
        return cursor.lastrowid


def save_git_pattern(session_id: int, pattern_type: str, commit_sha: str = "", commit_message: str = "", lines_changed: int = 0):
    """Save a git pattern from behavioral analysis."""
    with get_db() as conn:
        conn.execute(
            """INSERT INTO git_patterns (session_id, pattern_type, commit_sha, commit_message, lines_changed)
               VALUES (?, ?, ?, ?, ?)""",
            (session_id, pattern_type, commit_sha, commit_message, lines_changed)
        )


def flag_behavioral_pattern(session_id: int, pattern_name: str, evidence: str = "", severity: str = "medium"):
    """Flag a behavioral pattern - increment occurrence_count if exists, create if new."""
    now = datetime.now().isoformat()

    with get_db() as conn:
        # Try to get existing
        result = conn.execute(
            "SELECT occurrence_count FROM behavioral_patterns WHERE session_id = ? AND pattern_name = ?",
            (session_id, pattern_name)
        ).fetchone()

        if result:
            # Increment occurrence count
            conn.execute(
                """UPDATE behavioral_patterns
                   SET occurrence_count = occurrence_count + 1,
                       last_updated = ?,
                       evidence = ?,
                       severity = ?
                   WHERE session_id = ? AND pattern_name = ?""",
                (now, evidence, severity, session_id, pattern_name)
            )
        else:
            # Create new
            conn.execute(
                """INSERT INTO behavioral_patterns (session_id, pattern_name, evidence, severity, first_flagged, last_updated, occurrence_count)
                   VALUES (?, ?, ?, ?, ?, ?, 1)""",
                (session_id, pattern_name, evidence, severity, now, now)
            )


def track_recurring_issue(repo_path: str, issue_signature: str):
    """Track a recurring issue across scans - increment occurrence_count if exists."""
    now = datetime.now().isoformat()

    with get_db() as conn:
        # Try to get existing
        result = conn.execute(
            "SELECT occurrence_count FROM recurring_issues WHERE repo_path = ? AND issue_signature = ?",
            (repo_path, issue_signature)
        ).fetchone()

        if result:
            # Increment occurrence count
            conn.execute(
                """UPDATE recurring_issues
                   SET occurrence_count = occurrence_count + 1,
                       last_seen = ?
                   WHERE repo_path = ? AND issue_signature = ?""",
                (now, repo_path, issue_signature)
            )
        else:
            # Create new
            conn.execute(
                """INSERT INTO recurring_issues (repo_path, issue_signature, first_seen, last_seen, occurrence_count)
                   VALUES (?, ?, ?, ?, 1)""",
                (repo_path, issue_signature, now, now)
            )


def save_fix_attempt(issue_id: int, fix_prompt: str, success: bool, notes: str = ""):
    """Save a fix attempt for a security issue."""
    now = datetime.now().isoformat()

    with get_db() as conn:
        conn.execute(
            """INSERT INTO fix_attempts_roasting (issue_id, fix_prompt, attempted_at, success, notes)
               VALUES (?, ?, ?, ?, ?)""",
            (issue_id, fix_prompt, now, 1 if success else 0, notes)
        )


def get_scan_history(repo_path: str, limit: int = 10) -> List[Dict[str, Any]]:
    """Get scan history for a repository."""
    with get_db() as conn:
        scans = conn.execute(
            """SELECT id, repo_name, started_at, completed_at, total_issues, scan_duration_ms
               FROM scan_sessions
               WHERE repo_path = ?
               ORDER BY started_at DESC
               LIMIT ?""",
            (repo_path, limit)
        ).fetchall()

        return [dict(scan) for scan in scans]


def get_recurring_issues_for_repo(repo_path: str) -> List[Dict[str, Any]]:
    """Get all recurring issues for a repository."""
    with get_db() as conn:
        issues = conn.execute(
            """SELECT issue_signature, occurrence_count, first_seen, last_seen
               FROM recurring_issues
               WHERE repo_path = ?
               ORDER BY occurrence_count DESC""",
            (repo_path,)
        ).fetchall()

        return [dict(issue) for issue in issues]


def get_session_details(session_id: int) -> Dict[str, Any]:
    """
    Get comprehensive details for a scan session including all related data.

    Returns:
    - Session metadata
    - Security issues found
    - Git patterns detected
    - Behavioral patterns flagged
    """
    with get_db() as conn:
        # Get session metadata
        session = conn.execute(
            "SELECT * FROM scan_sessions WHERE id = ?",
            (session_id,)
        ).fetchone()

        if not session:
            return {}

        # Get security issues
        security_issues = conn.execute(
            """SELECT severity, category, summary, issue_url, fixed_at, fix_notes
               FROM security_issues
               WHERE session_id = ?
               ORDER BY severity DESC""",
            (session_id,)
        ).fetchall()

        # Get git patterns
        git_patterns = conn.execute(
            """SELECT pattern_type, commit_sha, commit_message, lines_changed
               FROM git_patterns
               WHERE session_id = ?""",
            (session_id,)
        ).fetchall()

        # Get behavioral patterns
        behavioral_patterns = conn.execute(
            """SELECT pattern_name, evidence, severity, occurrence_count, first_flagged, last_updated
               FROM behavioral_patterns
               WHERE session_id = ?
               ORDER BY occurrence_count DESC""",
            (session_id,)
        ).fetchall()

        return {
            "session": dict(session),
            "security_issues": [dict(issue) for issue in security_issues],
            "git_patterns": [dict(pattern) for pattern in git_patterns],
            "behavioral_patterns": [dict(pattern) for pattern in behavioral_patterns]
        }


# Initialize database on module import
initialize_database()
