// Codex ACP integration
// Spawns codex-acp subprocess and manages JSON-RPC communication

use anyhow::Result;

pub struct CodexAdapter;

impl CodexAdapter {
    pub fn new() -> Result<Self> {
        // TODO: Implement codex-acp integration
        Ok(Self)
    }

    pub async fn generate(&self, _prompt: String) -> Result<String> {
        // TODO: Implement prompt generation
        Ok("Placeholder observation".to_string())
    }
}
