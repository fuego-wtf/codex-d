// SQLite storage for observations

use anyhow::Result;
use crate::types::Observation;

pub struct Storage;

impl Storage {
    pub fn new() -> Result<Self> {
        // TODO: Initialize SQLite database
        Ok(Self)
    }

    pub fn save_observation(&self, _observation: &Observation) -> Result<()> {
        // TODO: Save to database
        Ok(())
    }

    pub fn load_all(&self) -> Result<Vec<Observation>> {
        // TODO: Load from database
        Ok(vec![])
    }
}
