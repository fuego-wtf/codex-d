// Codex ACP integration - spawns codex-acp subprocess and manages JSON-RPC communication

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::{Arc, Mutex};

use crate::types::{StreamEvent, ToolCallEvent, ToolCallLocation, ToolCallStatus, ToolCallUpdateEvent};

pub struct CodexAdapter {
    process: Arc<Mutex<Option<Child>>>,
    stdin: Arc<Mutex<Option<ChildStdin>>>,
    stdout: Arc<Mutex<Option<BufReader<ChildStdout>>>>,
    next_id: Arc<Mutex<u64>>,
    session_id: Arc<Mutex<Option<String>>>,
    mcp_server_process: Arc<Mutex<Option<Child>>>,
    repo_path: Arc<Mutex<Option<String>>>,
    system_prompt: Arc<Mutex<String>>,
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
            mcp_server_process: Arc::new(Mutex::new(None)),
            repo_path: Arc::new(Mutex::new(None)),
            system_prompt: Arc::new(Mutex::new(String::new())),
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
        let init_response = self.read_response()?;

        eprintln!("ACP initialized");
        eprintln!("Init response: {:?}", init_response);

        // Now authenticate - use ChatGPT method (no API key needed)
        self.authenticate()?;

        Ok(())
    }

    /// Authenticate with codex-acp using ChatGPT subscription
    fn authenticate(&self) -> Result<()> {
        let id = self.next_id();

        // Use chatgpt auth method - requires ChatGPT Plus/Pro subscription
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id,
            method: "agent/authenticate".to_string(),
            params: json!({
                "methodId": "chatgpt"
            }),
        };

        self.send_request(&request)?;
        let auth_response = self.read_response()?;

        eprintln!("Authentication response: {:?}", auth_response);

        Ok(())
    }

    /// Create a new session with system prompt and repository path
    pub fn create_session(&self, system_prompt: String, repo_path: String) -> Result<String> {
        // Store repo path for MCP server
        *self.repo_path.lock().unwrap() = Some(repo_path.clone());

        // Start MCP server first
        self.start_mcp_server()?;

        let id = self.next_id();

        // Use the dropped repository path as working directory (not the app's directory)
        let cwd = repo_path.clone();

        // Configure MCP server connection with HTTP transport (codex-acp only supports HTTP, not SSE)
        let mcp_servers = json!([
            {
                "name": "codex-psychology",
                "type": "http",
                "url": "http://127.0.0.1:52848/mcp",
                "headers": [
                    {
                        "name": "Accept",
                        "value": "application/json, text/event-stream"
                    }
                ]
            },
            {
                "name": "deepwiki",
                "type": "http",
                "url": "https://mcp.aci.dev/gateway/mcp?bundle_key=3Nhg7HK34j8ylWkv4uTeCssOKX3vdMxHfOuD",
                "headers": []
            }
        ]);

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id,
            method: "session/new".to_string(),
            params: json!({
                "cwd": cwd,
                "mcpServers": mcp_servers,
                "mode": "bypassPermissions",  // Skip permission prompts
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

        // Store system prompt for use with each message
        *self.system_prompt.lock().unwrap() = system_prompt;

        eprintln!("Session created: {}", session_id);

        Ok(session_id)
    }

    /// Start the MCP server for psychology analysis
    fn start_mcp_server(&self) -> Result<()> {
        let project_root = std::env!("CARGO_MANIFEST_DIR");
        let mcp_server_path = format!("{}/mcp-servers/mcp_codex_psychology", project_root);

        // Check if MCP server exists
        if !std::path::Path::new(&mcp_server_path).exists() {
            eprintln!("MCP server not found at: {}", mcp_server_path);
            return Ok(()); // Not an error - just no MCP
        }

        eprintln!("Starting MCP server at: {}", mcp_server_path);

        // Check if venv exists, create if it doesn't
        let venv_path = format!("{}/venv", mcp_server_path);
        let venv_python = format!("{}/bin/python3", venv_path);

        if !std::path::Path::new(&venv_path).exists() {
            eprintln!("Creating Python virtual environment...");

            // Find system python3
            let system_python = which::which("python3")
                .or_else(|_| which::which("python"))
                .map_err(|_| anyhow!("Python not found. Please install Python 3"))?
                .to_string_lossy()
                .to_string();

            // Create venv
            let venv_status = Command::new(&system_python)
                .arg("-m")
                .arg("venv")
                .arg("venv")
                .current_dir(&mcp_server_path)
                .status()
                .context("Failed to create virtual environment")?;

            if !venv_status.success() {
                return Err(anyhow!("Failed to create virtual environment"));
            }

            eprintln!("✅ Virtual environment created successfully");
        }

        // Use venv python
        let python_path = if std::path::Path::new(&venv_python).exists() {
            venv_python
        } else {
            return Err(anyhow!("Virtual environment was not created properly"));
        };

        eprintln!("Using Python: {}", python_path);

        // Install dependencies
        eprintln!("Installing MCP server dependencies...");
        let install_status = Command::new(&python_path)
            .arg("-m")
            .arg("pip")
            .arg("install")
            .arg("-q")
            .arg("-r")
            .arg("requirements.txt")
            .current_dir(&mcp_server_path)
            .status()
            .context("Failed to install MCP dependencies")?;

        if !install_status.success() {
            return Err(anyhow!("Failed to install MCP server dependencies"));
        }

        eprintln!("✅ MCP server dependencies installed successfully");

        // Start the MCP server with SSE transport
        let mut child = Command::new(&python_path)
            .arg("run_sse_server.py")
            .current_dir(&mcp_server_path)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to start MCP server")?;

        // Spawn thread to consume stdout
        if let Some(stdout) = child.stdout.take() {
            std::thread::spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines().flatten() {
                    eprintln!("[MCP] {}", line);
                }
            });
        }

        // Spawn thread to consume stderr
        if let Some(stderr) = child.stderr.take() {
            std::thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines().flatten() {
                    eprintln!("[MCP stderr] {}", line);
                }
            });
        }

        *self.mcp_server_process.lock().unwrap() = Some(child);

        // Wait a bit for server to start
        std::thread::sleep(std::time::Duration::from_secs(2));

        eprintln!("MCP server started successfully on port 52848");
        Ok(())
    }

    /// Send a message and stream responses via callback
    pub fn send_message<F>(&self, message: String, mut callback: F) -> Result<()>
    where
        F: FnMut(StreamEvent),
    {
        let session_id = self.session_id.lock().unwrap().clone()
            .ok_or_else(|| anyhow!("No active session"))?;

        // Get the system prompt to send with each message (like secretsoul-gpui)
        let system_prompt = self.system_prompt.lock().unwrap().clone();

        let id = self.next_id();

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id,
            method: "session/prompt".to_string(),
            params: json!({
                "sessionId": session_id,
                "systemPrompt": [
                    {
                        "type": "text",
                        "text": system_prompt,
                    }
                ],
                "prompt": [
                    {
                        "type": "text",
                        "text": message,
                    }
                ],
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
                            // Check the update type
                            if let Some(update) = params.get("update") {
                                if let Some(session_update) = update.get("sessionUpdate").and_then(|s| s.as_str()) {
                                    match session_update {
                                        "agent_message_chunk" => {
                                            // Extract text from content.text
                                            if let Some(text) = update.get("content")
                                                .and_then(|c| c.get("text"))
                                                .and_then(|t| t.as_str()) {
                                                callback(StreamEvent::MessageChunk(text.to_string()));
                                            }
                                        }
                                        "agent_thought_chunk" => {
                                            // Extract text from content.text
                                            if let Some(text) = update.get("content")
                                                .and_then(|c| c.get("text"))
                                                .and_then(|t| t.as_str()) {
                                                callback(StreamEvent::ThoughtChunk(text.to_string()));
                                            }
                                        }
                                        "tool_call" => {
                                            // Parse tool call event
                                            if let Some(tool_call_id) = update.get("toolCallId").and_then(|t| t.as_str()) {
                                                let title = update.get("title").and_then(|t| t.as_str()).unwrap_or("").to_string();
                                                let kind = update.get("kind").and_then(|k| k.as_str()).unwrap_or("").to_string();
                                                let status_str = update.get("status").and_then(|s| s.as_str()).unwrap_or("in_progress");
                                                let status = match status_str {
                                                    "completed" => ToolCallStatus::Completed,
                                                    "failed" => ToolCallStatus::Failed,
                                                    _ => ToolCallStatus::InProgress,
                                                };

                                                let mut locations = Vec::new();
                                                if let Some(locs) = update.get("locations").and_then(|l| l.as_array()) {
                                                    for loc in locs {
                                                        if let Some(path) = loc.get("path").and_then(|p| p.as_str()) {
                                                            locations.push(ToolCallLocation {
                                                                path: path.to_string(),
                                                            });
                                                        }
                                                    }
                                                }

                                                callback(StreamEvent::ToolCall(ToolCallEvent {
                                                    tool_call_id: tool_call_id.to_string(),
                                                    title,
                                                    kind,
                                                    status,
                                                    locations,
                                                    mcp_server: Some(crate::types::McpServerType::CodexPsychology), // TODO: Parse from tool name
                                                }));
                                            }
                                        }
                                        "tool_call_update" => {
                                            // Parse tool call update event
                                            if let Some(tool_call_id) = update.get("toolCallId").and_then(|t| t.as_str()) {
                                                let content = update.get("content")
                                                    .and_then(|c| c.as_array())
                                                    .and_then(|arr| arr.first())
                                                    .and_then(|item| item.get("content"))
                                                    .and_then(|c| c.get("text"))
                                                    .and_then(|t| t.as_str())
                                                    .map(|s| s.to_string());

                                                let status = update.get("status").and_then(|s| s.as_str()).map(|s| {
                                                    match s {
                                                        "completed" => ToolCallStatus::Completed,
                                                        "failed" => ToolCallStatus::Failed,
                                                        _ => ToolCallStatus::InProgress,
                                                    }
                                                });

                                                // Extract progress information if available
                                                let progress = update.get("progress")
                                                    .and_then(|p| p.as_f64())
                                                    .map(|p| p as f32);

                                                let progress_message = update.get("progressMessage")
                                                    .or_else(|| update.get("progress_message"))
                                                    .and_then(|m| m.as_str())
                                                    .map(|s| s.to_string());

                                                callback(StreamEvent::ToolCallUpdate(ToolCallUpdateEvent {
                                                    tool_call_id: tool_call_id.to_string(),
                                                    content,
                                                    status,
                                                    progress,
                                                    progress_message,
                                                }));
                                            }
                                        }
                                        _ => {
                                            // Ignore other update types for now
                                        }
                                    }
                                }
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
        // Check if codex-acp exists in submodule (release)
        let release_path = "./codex-acp/target/release/codex-acp";
        if std::path::Path::new(release_path).exists() {
            return Ok(release_path.to_string());
        }

        // Check if codex-acp exists in submodule (debug)
        let debug_path = "./codex-acp/target/debug/codex-acp";
        if std::path::Path::new(debug_path).exists() {
            return Ok(debug_path.to_string());
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

        // Kill the MCP server subprocess
        if let Some(mut mcp_child) = self.mcp_server_process.lock().unwrap().take() {
            let _ = mcp_child.kill();
            let _ = mcp_child.wait();
            eprintln!("MCP server stopped");
        }
    }
}
