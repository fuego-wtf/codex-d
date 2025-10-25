// Codex ACP integration - spawns codex-acp subprocess and manages JSON-RPC communication

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::{Arc, Mutex};

use crate::types::StreamEvent;

pub struct CodexAdapter {
    process: Arc<Mutex<Option<Child>>>,
    stdin: Arc<Mutex<Option<ChildStdin>>>,
    stdout: Arc<Mutex<Option<BufReader<ChildStdout>>>>,
    next_id: Arc<Mutex<u64>>,
    session_id: Arc<Mutex<Option<String>>>,
}

#[derive(Debug, Serialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: u64,
    method: String,
    params: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Option<u64>,
    result: Option<serde_json::Value>,
    error: Option<serde_json::Value>,
    method: Option<String>,
    params: Option<serde_json::Value>,
}

impl CodexAdapter {
    /// Create a new CodexAdapter instance
    pub fn new() -> Result<Self> {
        Ok(Self {
            process: Arc::new(Mutex::new(None)),
            stdin: Arc::new(Mutex::new(None)),
            stdout: Arc::new(Mutex::new(None)),
            next_id: Arc::new(Mutex::new(1)),
            session_id: Arc::new(Mutex::new(None)),
        })
    }

    /// Spawn the codex-acp subprocess
    pub fn spawn(&self) -> Result<()> {
        // Find codex-acp binary
        let codex_acp_path = Self::find_codex_acp()?;

        eprintln!("Spawning codex-acp at: {}", codex_acp_path);

        // Spawn subprocess
        let mut child = Command::new(&codex_acp_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .context("Failed to spawn codex-acp process")?;

        let stdin = child.stdin.take()
            .ok_or_else(|| anyhow!("Failed to get stdin"))?;
        let stdout = child.stdout.take()
            .ok_or_else(|| anyhow!("Failed to get stdout"))?;

        *self.process.lock().unwrap() = Some(child);
        *self.stdin.lock().unwrap() = Some(stdin);
        *self.stdout.lock().unwrap() = Some(BufReader::new(stdout));

        eprintln!("codex-acp spawned successfully");

        Ok(())
    }

    /// Initialize ACP connection
    pub fn initialize(&self) -> Result<()> {
        let id = self.next_id();

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id,
            method: "initialize".to_string(),
            params: json!({
                "protocolVersion": 1,
                "clientCapabilities": {
                    "fs": {
                        "read": true,
                        "write": true
                    },
                    "terminal": true
                }
            }),
        };

        self.send_request(&request)?;
        self.read_response()?;

        eprintln!("ACP initialized");

        Ok(())
    }

    /// Create a new session with system prompt
    pub fn create_session(&self, system_prompt: String) -> Result<String> {
        let id = self.next_id();

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id,
            method: "session/new".to_string(),
            params: json!({
                "systemPrompt": system_prompt,
            }),
        };

        self.send_request(&request)?;
        let response = self.read_response()?;

        let session_id = response
            .result
            .as_ref()
            .and_then(|r| r.get("sessionId"))
            .and_then(|s| s.as_str())
            .ok_or_else(|| anyhow!("No sessionId in response"))?
            .to_string();

        *self.session_id.lock().unwrap() = Some(session_id.clone());

        eprintln!("Session created: {}", session_id);

        Ok(session_id)
    }

    /// Send a message and stream responses via callback
    pub fn send_message<F>(&self, message: String, mut callback: F) -> Result<()>
    where
        F: FnMut(StreamEvent),
    {
        let session_id = self.session_id.lock().unwrap().clone()
            .ok_or_else(|| anyhow!("No active session"))?;

        let id = self.next_id();

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id,
            method: "session/prompt".to_string(),
            params: json!({
                "sessionId": session_id,
                "message": message,
            }),
        };

        self.send_request(&request)?;

        // Read streaming responses
        loop {
            let response = self.read_response()?;

            // Handle notifications (streaming updates)
            if let Some(method) = response.method {
                match method.as_str() {
                    "session/update" => {
                        if let Some(params) = response.params {
                            if let Some(content) = params.get("content").and_then(|c| c.as_str()) {
                                callback(StreamEvent::MessageChunk(content.to_string()));
                            }
                        }
                    }
                    "session/complete" => {
                        eprintln!("Stream complete");
                        break;
                    }
                    _ => {
                        eprintln!("Unknown method: {}", method);
                    }
                }
            }

            // Handle final response
            if response.id.is_some() && response.id.unwrap() == id {
                eprintln!("Received final response for request {}", id);
                break;
            }
        }

        Ok(())
    }

    fn send_request(&self, request: &JsonRpcRequest) -> Result<()> {
        let json = serde_json::to_string(request)?;
        let mut stdin = self.stdin.lock().unwrap();
        let stdin = stdin.as_mut()
            .ok_or_else(|| anyhow!("No stdin available"))?;

        writeln!(stdin, "{}", json)?;
        stdin.flush()?;

        eprintln!("-> {}", json);

        Ok(())
    }

    fn read_response(&self) -> Result<JsonRpcResponse> {
        let mut stdout = self.stdout.lock().unwrap();
        let stdout = stdout.as_mut()
            .ok_or_else(|| anyhow!("No stdout available"))?;

        let mut line = String::new();
        stdout.read_line(&mut line)?;

        if line.is_empty() {
            return Err(anyhow!("EOF reached"));
        }

        eprintln!("<- {}", line.trim());

        let response: JsonRpcResponse = serde_json::from_str(&line)
            .context("Failed to parse JSON-RPC response")?;

        Ok(response)
    }

    fn next_id(&self) -> u64 {
        let mut id = self.next_id.lock().unwrap();
        let current = *id;
        *id += 1;
        current
    }

    fn find_codex_acp() -> Result<String> {
        // Check if codex-acp exists in submodule
        let submodule_path = "./codex-acp/target/release/codex-acp";
        if std::path::Path::new(submodule_path).exists() {
            return Ok(submodule_path.to_string());
        }

        // Check in PATH
        which::which("codex-acp")
            .map(|p| p.display().to_string())
            .context("codex-acp not found. Build it with: cd codex-acp && cargo build --release")
    }
}

impl Drop for CodexAdapter {
    fn drop(&mut self) {
        if let Some(mut child) = self.process.lock().unwrap().take() {
            let _ = child.kill();
            let _ = child.wait();
            eprintln!("codex-acp process terminated");
        }
    }
}
