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
    LifecycleEvent(LifecycleEvent),
    PermissionRequest(PermissionRequest),
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
        }
    }

    pub fn running(tool_name: String) -> Self {
        Self {
            tool_name,
            status: LifecycleStatus::Running,
            timestamp: chrono::Utc::now().timestamp(),
            error: None,
        }
    }

    pub fn completed(tool_name: String) -> Self {
        Self {
            tool_name,
            status: LifecycleStatus::Completed,
            timestamp: chrono::Utc::now().timestamp(),
            error: None,
        }
    }

    pub fn failed(tool_name: String, error: String) -> Self {
        Self {
            tool_name,
            status: LifecycleStatus::Failed,
            timestamp: chrono::Utc::now().timestamp(),
            error: Some(error),
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
// Git Analysis
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitAnalysis {
    pub pattern_type: String,
    pub evidence: Vec<CommitEvidence>,
    pub summary: String,
    pub severity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitEvidence {
    pub sha: String,
    pub message: String,
    pub lines_changed: usize,
}
