// Core data types for codex-d

use serde::{Deserialize, Serialize};

// ============================================================================
// App State
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    AwaitingRepoSelection,
    Enriching,
    ChatActive,
}

// ============================================================================
// Streaming Events
// ============================================================================

#[derive(Debug, Clone)]
pub enum StreamEvent {
    MessageChunk(String),
    ThoughtChunk(String),
    ToolCall(ToolCallEvent),
    ToolCallUpdate(ToolCallUpdateEvent),
    LifecycleEvent(LifecycleEvent),
    PermissionRequest(PermissionRequest),
}

// ============================================================================
// MCP Server Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum McpServerType {
    #[serde(rename = "codex_psychology")]
    CodexPsychology,
    #[serde(rename = "aikido_scanner")]
    AikidoScanner,
    #[serde(rename = "kontext_dev")]
    KontextDev,
    #[serde(rename = "gate22_gateway")]
    Gate22Gateway,
}

impl McpServerType {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::CodexPsychology => "codex psychology",
            Self::AikidoScanner => "AikidoScanner",
            Self::KontextDev => "kontext.dev",
            Self::Gate22Gateway => "gate22 gateway",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::CodexPsychology => "🧠",
            Self::AikidoScanner => "🛡️",
            Self::KontextDev => "📚",
            Self::Gate22Gateway => "🌐",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerInfo {
    pub server_type: McpServerType,
    pub host: String,
    pub port: u16,
    pub status: McpServerStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum McpServerStatus {
    #[serde(rename = "connected")]
    Connected,
    #[serde(rename = "disconnected")]
    Disconnected,
    #[serde(rename = "connecting")]
    Connecting,
    #[serde(rename = "error")]
    Error,
}

// ============================================================================
// Tool Call Events
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallEvent {
    pub tool_call_id: String,
    pub title: String,
    pub kind: String,
    pub status: ToolCallStatus,
    pub locations: Vec<ToolCallLocation>,
    pub mcp_server: Option<McpServerType>, // Which MCP server is handling this tool
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallLocation {
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ToolCallStatus {
    #[serde(rename = "in_progress")]
    InProgress,
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "failed")]
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallUpdateEvent {
    pub tool_call_id: String,
    pub content: Option<String>,
    pub status: Option<ToolCallStatus>,
}

// ============================================================================
// Messages
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    User {
        content: String,
        timestamp: i64,
    },
    Assistant {
        content: String,
        timestamp: i64,
    },
}

impl Message {
    pub fn user(content: String) -> Self {
        Self::User {
            content,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    pub fn assistant(content: String) -> Self {
        Self::Assistant {
            content,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    pub fn content(&self) -> &str {
        match self {
            Self::User { content, .. } => content,
            Self::Assistant { content, .. } => content,
        }
    }

    pub fn timestamp(&self) -> i64 {
        match self {
            Self::User { timestamp, .. } => *timestamp,
            Self::Assistant { timestamp, .. } => *timestamp,
        }
    }

    pub fn is_user(&self) -> bool {
        matches!(self, Self::User { .. })
    }

    pub fn is_assistant(&self) -> bool {
        matches!(self, Self::Assistant { .. })
    }
}

// ============================================================================
// Tool Lifecycle Events
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleEvent {
    pub tool_name: String,
    pub status: LifecycleStatus,
    pub timestamp: i64,
    pub error: Option<String>,
    pub progress: Option<f32>,  // 0-100 percentage for Running status
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LifecycleStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

impl LifecycleEvent {
    pub fn pending(tool_name: String) -> Self {
        Self {
            tool_name,
            status: LifecycleStatus::Pending,
            timestamp: chrono::Utc::now().timestamp(),
            error: None,
            progress: None,
        }
    }

    pub fn running(tool_name: String) -> Self {
        Self {
            tool_name,
            status: LifecycleStatus::Running,
            timestamp: chrono::Utc::now().timestamp(),
            error: None,
            progress: Some(0.0),
        }
    }

    pub fn progress(tool_name: String, progress: f32) -> Self {
        Self {
            tool_name,
            status: LifecycleStatus::Running,
            timestamp: chrono::Utc::now().timestamp(),
            error: None,
            progress: Some(progress.clamp(0.0, 100.0)),
        }
    }

    pub fn completed(tool_name: String) -> Self {
        Self {
            tool_name,
            status: LifecycleStatus::Completed,
            timestamp: chrono::Utc::now().timestamp(),
            error: None,
            progress: Some(100.0),
        }
    }

    pub fn failed(tool_name: String, error: String) -> Self {
        Self {
            tool_name,
            status: LifecycleStatus::Failed,
            timestamp: chrono::Utc::now().timestamp(),
            error: Some(error),
            progress: None,
        }
    }
}

// ============================================================================
// Permission Requests
// ============================================================================

#[derive(Debug, Clone)]
pub struct PermissionRequest {
    pub id: String,
    pub tool_name: String,
    pub description: String,
}

// ============================================================================
// Timeline Events (for chronological trajectory display)
// ============================================================================

#[derive(Debug, Clone)]
pub enum TimelineEvent {
    UserMessage {
        content: String,
        timestamp: i64,
    },
    Thought {
        content: String,
        timestamp: i64,
    },
    ToolCall {
        tool_call_id: String,
        title: String,
        kind: String,
        status: ToolCallStatus,
        locations: Vec<ToolCallLocation>,
        output: Option<String>,
        timestamp: i64,
        mcp_server: Option<McpServerType>, // For transparency: which MCP server
        routed_via: Option<McpServerType>,  // If routed through gate22 gateway
    },
    AssistantMessage {
        content: String,
        timestamp: i64,
    },
    McpServerConnected {
        server_type: McpServerType,
        host: String,
        port: u16,
        timestamp: i64,
    },
    McpServerDisconnected {
        server_type: McpServerType,
        reason: Option<String>,
        timestamp: i64,
    },
    AgentFixPrompt {
        prompt: String,
        context: Option<String>,
        timestamp: i64,
    },
    SecurityFinding {
        vulnerability_id: String,
        severity: String,  // "critical", "high", "medium", "low"
        title: String,
        description: String,
        file_path: String,
        line_number: Option<u32>,
        cwe_id: Option<String>,
        recommendation: String,
        timestamp: i64,
    },
}

impl TimelineEvent {
    pub fn timestamp(&self) -> i64 {
        match self {
            Self::UserMessage { timestamp, .. } => *timestamp,
            Self::Thought { timestamp, .. } => *timestamp,
            Self::ToolCall { timestamp, .. } => *timestamp,
            Self::AssistantMessage { timestamp, .. } => *timestamp,
            Self::McpServerConnected { timestamp, .. } => *timestamp,
            Self::McpServerDisconnected { timestamp, .. } => *timestamp,
            Self::AgentFixPrompt { timestamp, .. } => *timestamp,
            Self::SecurityFinding { timestamp, .. } => *timestamp,
        }
    }
}

// ============================================================================
// Git Analysis
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitAnalysis {
    pub patterns: Vec<GitPattern>,
    pub summary: String,
    pub total_commits_analyzed: usize,
    pub severity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitPattern {
    pub pattern_type: String,
    pub title: String,
    pub description: String,
    pub evidence: Vec<CommitEvidence>,
    pub severity: f32,
    pub insight: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitEvidence {
    pub sha: String,
    pub message: String,
    pub lines_changed: usize,
}
