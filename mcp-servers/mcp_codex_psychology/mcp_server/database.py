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


# Initialize database on module import
initialize_database()
