// Git repository analysis - detects behavioral patterns in commit history

use anyhow::{Context, Result};
use git2::{DiffOptions, Repository, Time};
use std::collections::HashSet;
use std::path::Path;
use chrono::{DateTime, Datelike, Timelike, Utc};

use crate::types::{CommitEvidence, GitAnalysis, GitPattern};

pub struct GitAnalyzer;

#[derive(Debug)]
struct CommitData {
    sha: String,
    message: String,
    lines_changed: usize,
    timestamp: i64,
    hour: u32,
    day_of_week: u32,
    files_changed: Vec<String>,
}

impl GitAnalyzer {
    /// Analyze a git repository for behavioral patterns
    pub async fn analyze<F>(repo_path: impl AsRef<Path>, progress_callback: F) -> Result<GitAnalysis>
    where
        F: Fn(String, f32) + Send + 'static,
    {
        let repo_path = repo_path.as_ref().to_path_buf();

        // Run blocking git operations in background thread
        tokio::task::spawn_blocking(move || {
            Self::analyze_blocking(&repo_path, progress_callback)
        })
        .await
        .context("Git analyzer task failed")?
    }

    fn analyze_blocking<F>(repo_path: &Path, progress_cb: F) -> Result<GitAnalysis>
    where
        F: Fn(String, f32),
    {
        progress_cb("Opening repository".to_string(), 5.0);
        let repo = Repository::open(repo_path)
            .context("Failed to open git repository")?;

        // Collect commit data
        progress_cb("Collecting commits".to_string(), 15.0);
        let commits = Self::collect_commits(&repo, 100)?;

        if commits.is_empty() {
            progress_cb("Analysis complete".to_string(), 100.0);
            return Ok(GitAnalysis {
                patterns: vec![],
                summary: "No commits found in repository".to_string(),
                total_commits_analyzed: 0,
                severity: 0.0,
            });
        }

        progress_cb("Building commit dataset".to_string(), 25.0);

        // Detect all patterns
        let mut patterns = Vec::new();

        // 1. Minimizing Language Pattern
        progress_cb("Detecting minimizing language".to_string(), 35.0);
        if let Some(pattern) = Self::detect_minimizing_language(&commits) {
            patterns.push(pattern);
        }

        // 2. Commitment Issues (commit size distribution)
        progress_cb("Analyzing commit sizes".to_string(), 50.0);
        if let Some(pattern) = Self::detect_commitment_issues(&commits) {
            patterns.push(pattern);
        }

        // 3. Temporal Patterns (late night, weekend work)
        progress_cb("Detecting temporal patterns".to_string(), 65.0);
        if let Some(pattern) = Self::detect_temporal_patterns(&commits) {
            patterns.push(pattern);
        }

        // 4. Self-Deception (message vs. actual changes)
        progress_cb("Analyzing self-deception".to_string(), 75.0);
        if let Some(pattern) = Self::detect_self_deception(&commits) {
            patterns.push(pattern);
        }

        // 5. File Avoidance
        progress_cb("Detecting file avoidance".to_string(), 85.0);
        if let Some(pattern) = Self::detect_file_avoidance(&commits, &repo)? {
            patterns.push(pattern);
        }

        // Calculate overall severity
        progress_cb("Calculating severity".to_string(), 95.0);
        let severity = if patterns.is_empty() {
            0.0
        } else {
            patterns.iter().map(|p| p.severity).sum::<f32>() / patterns.len() as f32
        };

        // Generate summary
        let summary = Self::generate_summary(&patterns, commits.len());

        progress_cb("Analysis complete".to_string(), 100.0);

        Ok(GitAnalysis {
            patterns,
            summary,
            total_commits_analyzed: commits.len(),
            severity,
        })
    }

    fn collect_commits(repo: &Repository, limit: usize) -> Result<Vec<CommitData>> {
        let mut revwalk = repo.revwalk()
            .context("Failed to create revwalk")?;

        revwalk.push_head()
            .context("Failed to push HEAD")?;

        let mut commits = Vec::new();

        for (idx, commit_id) in revwalk.enumerate() {
            if idx >= limit {
                break;
            }

            let commit_id = commit_id.context("Invalid commit ID")?;
            let commit = repo.find_commit(commit_id)
                .context("Failed to find commit")?;

            let message = commit.summary().unwrap_or("").to_string();
            let lines_changed = Self::get_commit_stats(&repo, &commit)?;
            let time = commit.time();

            // Convert to DateTime for analysis
            let dt = Self::git_time_to_datetime(&time);

            // Get files changed
            let files_changed = Self::get_changed_files(&repo, &commit)?;

            commits.push(CommitData {
                sha: commit_id.to_string()[..7].to_string(),
                message,
                lines_changed,
                timestamp: time.seconds(),
                hour: dt.hour(),
                day_of_week: dt.weekday().num_days_from_monday(),
                files_changed,
            });
        }

        Ok(commits)
    }

    fn detect_minimizing_language(commits: &[CommitData]) -> Option<GitPattern> {
        let minimizing_keywords = [
            "quick", "small", "tiny", "just", "minor", "simple",
            "oops", "fix typo", "small change", "quick fix",
        ];

        let mut evidence = Vec::new();

        for commit in commits {
            let message_lower = commit.message.to_lowercase();
            let has_minimizing = minimizing_keywords.iter()
                .any(|keyword| message_lower.contains(keyword));

            // Minimizing language + substantial change (>100 lines)
            if has_minimizing && commit.lines_changed > 100 {
                evidence.push(CommitEvidence {
                    sha: commit.sha.clone(),
                    message: commit.message.clone(),
                    lines_changed: commit.lines_changed,
                });
            }
        }

        if evidence.is_empty() {
            return None;
        }

        let severity = (evidence.len() as f32 / commits.len() as f32).min(1.0);

        Some(GitPattern {
            pattern_type: "minimizing_language".to_string(),
            title: "Minimizing Language in Commit Messages".to_string(),
            description: format!(
                "Found {} commits using minimizing language ('quick', 'small', 'just') while making substantial changes (>100 lines). \
                This pattern suggests downplaying work to manage expectations or reduce perceived risk.",
                evidence.len()
            ),
            evidence,
            severity,
            insight: "You're using self-protective language to minimize the perceived scope of your work. \
                What would it feel like to fully acknowledge the complexity of what you're building?".to_string(),
        })
    }

    fn detect_commitment_issues(commits: &[CommitData]) -> Option<GitPattern> {
        if commits.len() < 10 {
            return None; // Need enough data
        }

        // Analyze commit size distribution
        let avg_size: usize = commits.iter().map(|c| c.lines_changed).sum::<usize>() / commits.len();
        let small_commits = commits.iter().filter(|c| c.lines_changed < 20).count();
        let large_commits = commits.iter().filter(|c| c.lines_changed > 200).count();

        let small_ratio = small_commits as f32 / commits.len() as f32;
        let large_ratio = large_commits as f32 / commits.len() as f32;

        // Pattern: Either lots of tiny commits OR rare massive commits
        let has_pattern = small_ratio > 0.6 || large_ratio > 0.3;

        if !has_pattern {
            return None;
        }

        let evidence: Vec<CommitEvidence> = if small_ratio > 0.6 {
            // Frequent tiny commits
            commits.iter()
                .filter(|c| c.lines_changed < 20)
                .take(5)
                .map(|c| CommitEvidence {
                    sha: c.sha.clone(),
                    message: c.message.clone(),
                    lines_changed: c.lines_changed,
                })
                .collect()
        } else {
            // Rare massive commits
            commits.iter()
                .filter(|c| c.lines_changed > 200)
                .take(5)
                .map(|c| CommitEvidence {
                    sha: c.sha.clone(),
                    message: c.message.clone(),
                    lines_changed: c.lines_changed,
                })
                .collect()
        };

        let (description, insight) = if small_ratio > 0.6 {
            (
                format!(
                    "{}% of commits are tiny (<20 lines). Average commit size: {} lines. \
                    This suggests either hyper-vigilance about commit frequency or fear of making substantial changes.",
                    (small_ratio * 100.0) as u32, avg_size
                ),
                "Frequent tiny commits can indicate perfectionism or fear of 'breaking things'. \
                What would happen if you bundled related changes into more meaningful commits?".to_string()
            )
        } else {
            (
                format!(
                    "{}% of commits are massive (>200 lines). Average commit size: {} lines. \
                    This suggests batch-saving work or avoiding commits until changes feel 'complete'.",
                    (large_ratio * 100.0) as u32, avg_size
                ),
                "Large commits make it hard to review history and revert changes. \
                What prevents you from committing more incrementally?".to_string()
            )
        };

        Some(GitPattern {
            pattern_type: "commitment_issues".to_string(),
            title: "Commit Size Distribution Pattern".to_string(),
            description,
            evidence,
            severity: small_ratio.max(large_ratio),
            insight,
        })
    }

    fn detect_temporal_patterns(commits: &[CommitData]) -> Option<GitPattern> {
        let late_night_commits = commits.iter()
            .filter(|c| c.hour >= 22 || c.hour <= 5)
            .count();

        let weekend_commits = commits.iter()
            .filter(|c| c.day_of_week >= 5) // Saturday=5, Sunday=6
            .count();

        let late_night_ratio = late_night_commits as f32 / commits.len() as f32;
        let weekend_ratio = weekend_commits as f32 / commits.len() as f32;

        // Pattern detected if >30% late night OR >25% weekend
        if late_night_ratio < 0.3 && weekend_ratio < 0.25 {
            return None;
        }

        let evidence: Vec<CommitEvidence> = commits.iter()
            .filter(|c| (c.hour >= 22 || c.hour <= 5) || c.day_of_week >= 5)
            .take(5)
            .map(|c| CommitEvidence {
                sha: c.sha.clone(),
                message: c.message.clone(),
                lines_changed: c.lines_changed,
            })
            .collect();

        let description = if late_night_ratio > weekend_ratio {
            format!(
                "{}% of commits happen between 10pm-5am. This suggests late-night coding sessions, \
                potentially driven by deadlines, flow states, or avoiding daytime interruptions.",
                (late_night_ratio * 100.0) as u32
            )
        } else {
            format!(
                "{}% of commits happen on weekends. This suggests weekend work, potentially \
                to catch up, avoid interruptions, or due to passion for the project.",
                (weekend_ratio * 100.0) as u32
            )
        };

        Some(GitPattern {
            pattern_type: "temporal_imbalance".to_string(),
            title: "Work-Life Balance Pattern".to_string(),
            description,
            evidence,
            severity: late_night_ratio.max(weekend_ratio),
            insight: "Irregular work hours can indicate passion, but also burnout risk. \
                What boundaries would help you sustain this work long-term?".to_string(),
        })
    }

    fn detect_self_deception(commits: &[CommitData]) -> Option<GitPattern> {
        let fix_keywords = ["fix", "bugfix", "patch", "hotfix", "repair"];
        let refactor_keywords = ["refactor", "cleanup", "reorganize", "restructure"];

        let mut evidence = Vec::new();

        for commit in commits {
            let message_lower = commit.message.to_lowercase();

            // "fix" but massive change (>300 lines) = likely more than a fix
            let is_fix_claim = fix_keywords.iter().any(|k| message_lower.contains(k));
            if is_fix_claim && commit.lines_changed > 300 {
                evidence.push(CommitEvidence {
                    sha: commit.sha.clone(),
                    message: commit.message.clone(),
                    lines_changed: commit.lines_changed,
                });
            }

            // "cleanup" but massive change = likely rewrite
            let is_cleanup_claim = refactor_keywords.iter().any(|k| message_lower.contains(k));
            if is_cleanup_claim && commit.lines_changed > 400 {
                evidence.push(CommitEvidence {
                    sha: commit.sha.clone(),
                    message: commit.message.clone(),
                    lines_changed: commit.lines_changed,
                });
            }
        }

        if evidence.is_empty() {
            return None;
        }

        let severity = (evidence.len() as f32 / commits.len() as f32).min(0.8);

        Some(GitPattern {
            pattern_type: "self_deception".to_string(),
            title: "Message vs. Reality Mismatch".to_string(),
            description: format!(
                "Found {} commits where the message ('fix', 'cleanup') doesn't match the scope (>300 lines changed). \
                This suggests minimizing the impact of changes to yourself or others.",
                evidence.len()
            ),
            evidence,
            severity,
            insight: "When commit messages don't reflect reality, it creates cognitive dissonance. \
                What would honest commit messages reveal about your work?".to_string(),
        })
    }

    fn detect_file_avoidance(commits: &[CommitData], repo: &Repository) -> Result<Option<GitPattern>> {
        // Get all files in repo
        let head = repo.head()?;
        let tree = head.peel_to_tree()?;

        let mut all_files = HashSet::new();
        tree.walk(git2::TreeWalkMode::PreOrder, |_, entry| {
            if entry.kind() == Some(git2::ObjectType::Blob) {
                if let Some(name) = entry.name() {
                    all_files.insert(name.to_string());
                }
            }
            0
        })?;

        // Get touched files from commits
        let mut touched_files = HashSet::new();
        for commit in commits {
            for file in &commit.files_changed {
                touched_files.insert(file.clone());
            }
        }

        // Find files never touched
        let untouched: Vec<String> = all_files.difference(&touched_files)
            .take(10)
            .cloned()
            .collect();

        if untouched.len() < 5 {
            return Ok(None); // Not enough untouched files to be meaningful
        }

        let avoidance_ratio = untouched.len() as f32 / all_files.len() as f32;

        if avoidance_ratio < 0.2 {
            return Ok(None); // Less than 20% untouched is normal
        }

        Ok(Some(GitPattern {
            pattern_type: "file_avoidance".to_string(),
            title: "File Avoidance Pattern".to_string(),
            description: format!(
                "{}% of files ({}/{}) haven't been touched in the last {} commits. \
                Examples: {}. This could indicate technical debt accumulation or avoiding complex areas.",
                (avoidance_ratio * 100.0) as u32,
                untouched.len(),
                all_files.len(),
                commits.len(),
                untouched.iter().take(5).cloned().collect::<Vec<_>>().join(", ")
            ),
            evidence: vec![], // No specific commits, but pattern exists
            severity: avoidance_ratio.min(0.7),
            insight: "Avoided files often represent technical debt or areas of uncertainty. \
                What would it take to confidently work on these files?".to_string(),
        }))
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

    fn get_changed_files(repo: &Repository, commit: &git2::Commit) -> Result<Vec<String>> {
        let current_tree = commit.tree()?;
        let parent_tree = if commit.parent_count() > 0 {
            Some(commit.parent(0)?.tree()?)
        } else {
            None
        };

        let diff = repo.diff_tree_to_tree(
            parent_tree.as_ref(),
            Some(&current_tree),
            None,
        )?;

        let mut files = Vec::new();
        diff.foreach(
            &mut |delta, _| {
                if let Some(path) = delta.new_file().path() {
                    files.push(path.to_string_lossy().to_string());
                }
                true
            },
            None, None, None,
        )?;

        Ok(files)
    }

    fn git_time_to_datetime(time: &Time) -> DateTime<Utc> {
        DateTime::from_timestamp(time.seconds(), 0).unwrap_or_else(|| Utc::now())
    }

    fn generate_summary(patterns: &[GitPattern], total_commits: usize) -> String {
        if patterns.is_empty() {
            return format!("Analyzed {} commits. No significant behavioral patterns detected.", total_commits);
        }

        let pattern_titles: Vec<&str> = patterns.iter()
            .map(|p| p.title.as_str())
            .collect();

        format!(
            "Analyzed {} commits and detected {} behavioral patterns: {}",
            total_commits,
            patterns.len(),
            pattern_titles.join(", ")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_analyzer_on_valid_repo() {
        // This test requires a valid git repo
        let result = GitAnalyzer::analyze(".").await;

        match result {
            Ok(analysis) => {
                println!("Analysis: {}", analysis.summary);
                println!("Patterns found: {}", analysis.patterns.len());
                for pattern in &analysis.patterns {
                    println!("  - {}: {}", pattern.title, pattern.description);
                }
            }
            Err(e) => {
                println!("Expected error (not a git repo?): {}", e);
            }
        }
    }
}
