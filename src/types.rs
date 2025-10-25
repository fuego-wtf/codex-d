// Core data types for codex-d

use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    pub id: String,
    pub repo_path: String,
    pub narrative: String,
    pub git_analysis: GitAnalysis,
    pub timestamp: i64,
}
