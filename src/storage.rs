// SQLite storage for conversation history and observation tracking

use anyhow::{Context, Result};
use rusqlite::{Connection, params};
use std::path::PathBuf;

use crate::types::{Message, GitAnalysis};

pub struct Storage {
    conn: Connection,
}

impl Storage {
    /// Create new storage instance with database at given path
    pub fn new(db_path: &str) -> Result<Self> {
        let path = PathBuf::from(db_path);

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .context("Failed to create database directory")?;
        }

        let conn = Connection::open(&path)
            .context("Failed to open database")?;

        let storage = Self { conn };
        storage.initialize_schema()?;

        eprintln!("Storage initialized at: {:?}", path);

        Ok(storage)
    }

    fn initialize_schema(&self) -> Result<()> {
        // Messages table
        self.conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                timestamp INTEGER NOT NULL
            )
            "#,
            [],
        ).context("Failed to create messages table")?;

        // Observations table (for longitudinal tracking)
        self.conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS observations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                repo_path TEXT NOT NULL,
                observation TEXT NOT NULL,
                patterns_summary TEXT NOT NULL,
                total_commits INTEGER NOT NULL,
                severity REAL NOT NULL,
                timestamp INTEGER NOT NULL
            )
            "#,
            [],
        ).context("Failed to create observations table")?;

        // Index for efficient repo lookups
        self.conn.execute(
            r#"
            CREATE INDEX IF NOT EXISTS idx_observations_repo
            ON observations(repo_path, timestamp DESC)
            "#,
            [],
        ).context("Failed to create observations index")?;

        Ok(())
    }

    /// Save a message to the database
    pub fn save_message(&self, message: &Message) -> Result<()> {
        let (role, content, timestamp) = match message {
            Message::User { content, timestamp } => ("user", content, timestamp),
            Message::Assistant { content, timestamp } => ("assistant", content, timestamp),
        };

        self.conn.execute(
            "INSERT INTO messages (role, content, timestamp) VALUES (?1, ?2, ?3)",
            params![role, content, timestamp],
        ).context("Failed to save message")?;

        Ok(())
    }

    /// Load all messages from the database
    pub fn load_messages(&self) -> Result<Vec<Message>> {
        let mut stmt = self.conn.prepare(
            "SELECT role, content, timestamp FROM messages ORDER BY timestamp ASC"
        ).context("Failed to prepare query")?;

        let messages = stmt.query_map([], |row| {
            let role: String = row.get(0)?;
            let content: String = row.get(1)?;
            let timestamp: i64 = row.get(2)?;

            Ok(match role.as_str() {
                "user" => Message::User { content, timestamp },
                "assistant" => Message::Assistant { content, timestamp },
                _ => Message::User { content: format!("Unknown role: {}", role), timestamp },
            })
        })?.collect::<Result<Vec<_>, _>>()
            .context("Failed to parse messages")?;

        Ok(messages)
    }

    /// Delete all messages (panic wipe)
    pub fn panic_wipe(&self) -> Result<()> {
        self.conn.execute("DELETE FROM messages", [])
            .context("Failed to delete messages")?;

        // Vacuum to reclaim space
        self.conn.execute("VACUUM", [])
            .context("Failed to vacuum database")?;

        eprintln!("Database wiped");

        Ok(())
    }

    /// Get message count
    pub fn count_messages(&self) -> Result<usize> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM messages",
            [],
            |row| row.get(0)
        ).context("Failed to count messages")?;

        Ok(count as usize)
    }

    /// Save an observation for longitudinal tracking
    pub fn save_observation(
        &self,
        repo_path: &str,
        observation: &str,
        analysis: &GitAnalysis,
    ) -> Result<()> {
        let patterns_summary = analysis.patterns.iter()
            .map(|p| format!("• {}: {}", p.title, p.description))
            .collect::<Vec<_>>()
            .join("\n");

        self.conn.execute(
            r#"
            INSERT INTO observations
            (repo_path, observation, patterns_summary, total_commits, severity, timestamp)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
            params![
                repo_path,
                observation,
                patterns_summary,
                analysis.total_commits_analyzed,
                analysis.severity,
                chrono::Utc::now().timestamp(),
            ],
        ).context("Failed to save observation")?;

        Ok(())
    }

    /// Load past observations for a repository (most recent first)
    pub fn load_observations(&self, repo_path: &str, limit: usize) -> Result<Vec<Observation>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT observation, patterns_summary, total_commits, severity, timestamp
            FROM observations
            WHERE repo_path = ?1
            ORDER BY timestamp DESC
            LIMIT ?2
            "#
        ).context("Failed to prepare observations query")?;

        let observations = stmt.query_map(params![repo_path, limit], |row| {
            Ok(Observation {
                observation: row.get(0)?,
                patterns_summary: row.get(1)?,
                total_commits: row.get(2)?,
                severity: row.get(3)?,
                timestamp: row.get(4)?,
            })
        })?.collect::<Result<Vec<_>, _>>()
            .context("Failed to parse observations")?;

        Ok(observations)
    }
}

/// Historical observation for longitudinal tracking
#[derive(Debug, Clone)]
pub struct Observation {
    pub observation: String,
    pub patterns_summary: String,
    pub total_commits: i64,
    pub severity: f64,
    pub timestamp: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_roundtrip() {
        // Use in-memory database for testing
        let storage = Storage::new(":memory:").unwrap();

        // Save some messages
        let msg1 = Message::user("Hello".to_string());
        let msg2 = Message::assistant("Hi there!".to_string());

        storage.save_message(&msg1).unwrap();
        storage.save_message(&msg2).unwrap();

        // Load and verify
        let messages = storage.load_messages().unwrap();
        assert_eq!(messages.len(), 2);
        assert!(messages[0].is_user());
        assert!(messages[1].is_assistant());
        assert_eq!(messages[0].content(), "Hello");
        assert_eq!(messages[1].content(), "Hi there!");

        // Test count
        assert_eq!(storage.count_messages().unwrap(), 2);

        // Test panic wipe
        storage.panic_wipe().unwrap();
        assert_eq!(storage.count_messages().unwrap(), 0);
    }
}
