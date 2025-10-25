// SQLite storage for conversations

use anyhow::Result;
use crate::types::Message;

pub struct Storage;

impl Storage {
    pub fn new() -> Result<Self> {
        // TODO: Initialize SQLite database
        Ok(Self)
    }

    pub fn save_message(&self, _message: &Message) -> Result<()> {
        // TODO: Save to database
        Ok(())
    }

    pub fn load_messages(&self) -> Result<Vec<Message>> {
        // TODO: Load from database
        Ok(vec![])
    }

    pub fn panic_wipe(&self) -> Result<()> {
        // TODO: Delete all data
        Ok(())
    }
}
