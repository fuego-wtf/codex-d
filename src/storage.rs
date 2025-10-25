// SQLite storage for conversation history

use anyhow::{Context, Result};
use rusqlite::{Connection, params};
use std::path::PathBuf;

use crate::types::Message;

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
