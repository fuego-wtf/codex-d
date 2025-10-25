// Git repository analysis - detects behavioral patterns in commit history

use anyhow::{Context, Result};
use git2::{DiffOptions, Repository};
use std::path::Path;

use crate::types::{CommitEvidence, GitAnalysis};

pub struct GitAnalyzer;

impl GitAnalyzer {
    /// Analyze a git repository for behavioral patterns
    pub async fn analyze(repo_path: impl AsRef<Path>) -> Result<GitAnalysis> {
        let repo_path = repo_path.as_ref().to_path_buf();

        // Run blocking git operations in background thread
        tokio::task::spawn_blocking(move || {
            Self::analyze_blocking(&repo_path)
        })
        .await
        .context("Git analyzer task failed")?
    }

    fn analyze_blocking(repo_path: &Path) -> Result<GitAnalysis> {
        let repo = Repository::open(repo_path)
            .context("Failed to open git repository")?;

        let mut revwalk = repo.revwalk()
            .context("Failed to create revwalk")?;

        revwalk.push_head()
            .context("Failed to push HEAD")?;

        let mut evidence = Vec::new();
        let minimizing_keywords = ["quick", "small", "tiny", "just", "minor", "simple"];

        // Analyze last 50 commits
        for (idx, commit_id) in revwalk.enumerate() {
            if idx >= 50 {
                break;
            }

            let commit_id = commit_id.context("Invalid commit ID")?;
            let commit = repo.find_commit(commit_id)
                .context("Failed to find commit")?;

            let message = commit.message().unwrap_or("").to_lowercase();

            // Calculate diff stats
            let lines_changed = Self::get_commit_stats(&repo, &commit)?;

            // Detect minimizing language + large changes pattern
            let has_minimizing_lang = minimizing_keywords.iter()
                .any(|keyword| message.contains(keyword));

            if has_minimizing_lang && lines_changed > 100 {
                evidence.push(CommitEvidence {
                    sha: commit_id.to_string()[..7].to_string(),
                    message: commit.summary().unwrap_or("").to_string(),
                    lines_changed,
                });
            }
        }

        let severity = if evidence.len() > 10 {
            0.8
        } else if evidence.len() > 5 {
            0.6
        } else if evidence.len() > 2 {
            0.4
        } else {
            0.2
        };

        let summary = if !evidence.is_empty() {
            format!(
                "Found {} commits with minimizing language but >100 lines changed",
                evidence.len()
            )
        } else {
            "No significant message/size mismatch pattern detected".to_string()
        };

        Ok(GitAnalysis {
            pattern_type: "message_size_mismatch".to_string(),
            evidence,
            summary,
            severity,
        })
    }

    fn get_commit_stats(repo: &Repository, commit: &git2::Commit) -> Result<usize> {
        let current_tree = commit.tree()
            .context("Failed to get commit tree")?;

        let parent_tree = if commit.parent_count() > 0 {
            Some(commit.parent(0)
                .context("Failed to get parent commit")?
                .tree()
                .context("Failed to get parent tree")?)
        } else {
            None
        };

        let diff = repo.diff_tree_to_tree(
            parent_tree.as_ref(),
            Some(&current_tree),
            Some(&mut DiffOptions::new()),
        ).context("Failed to create diff")?;

        let stats = diff.stats()
            .context("Failed to get diff stats")?;

        Ok(stats.insertions() + stats.deletions())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_analyzer_on_valid_repo() {
        // This test requires a valid git repo - skip if not in one
        let result = GitAnalyzer::analyze(".").await;

        // Should either succeed or fail with clear error
        match result {
            Ok(analysis) => {
                assert_eq!(analysis.pattern_type, "message_size_mismatch");
                println!("Analysis: {} (severity: {})", analysis.summary, analysis.severity);
            }
            Err(e) => {
                println!("Expected error (not a git repo?): {}", e);
            }
        }
    }
}
