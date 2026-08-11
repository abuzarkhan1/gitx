//! Bug and regression history (docs/10 §9).
//!
//! Builds on fix/revert classification to surface *recurring problem areas* —
//! files repeatedly involved in fix-classified commits, reverts that follow
//! shortly after the change they revert, and areas whose fix density is high
//! relative to overall activity. These are evidence lists to guide
//! investigation, never predictions. All classification is heuristic and is
//! labeled as such.

use crate::classification::classify_commit_message;
use gitx_core::types::CommitClassification;
use gitx_git::Repository;
use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;

/// A file that has been repeatedly involved in fix/regression work.
#[derive(Debug, Clone, Serialize)]
pub struct ProblemFile {
    pub path: PathBuf,
    /// Number of fix-classified commits touching this file.
    pub fix_commits: u32,
    /// Total commits touching this file.
    pub total_changes: u32,
    /// fix_commits / total_changes, 0–1 (fix density).
    pub fix_density: f64,
    /// Number of reverts that touched this file.
    pub reverts: u32,
    /// Unix timestamps of the fix commits (for evidence).
    pub fix_times: Vec<i64>,
}

/// A revert commit and the change it (likely) reverted, when the reverted
/// commit is found among the file's recent history.
#[derive(Debug, Clone, Serialize)]
pub struct RevertPair {
    /// OID of the revert commit.
    pub revert_oid: String,
    /// OID of the reverted commit (best-effort, from the message).
    pub reverted_oid: Option<String>,
    /// Files the revert touched.
    pub paths: Vec<PathBuf>,
    /// Seconds between the reverted change and the revert (when known).
    pub gap_seconds: Option<i64>,
}

/// Full regression report.
#[derive(Debug, Clone, Default, Serialize)]
pub struct RegressionReport {
    /// Files with the highest fix density / recurring fix involvement.
    pub problem_files: Vec<ProblemFile>,
    /// Revert commits with their best-effort reverted counterpart.
    pub reverts: Vec<RevertPair>,
    /// Total commits analyzed and how many were fix-classified.
    pub total_commits: u32,
    pub total_fixes: u32,
    pub total_reverts: u32,
}

/// Walk the mainline from HEAD and compute the regression report (docs/10 §9).
/// Deterministic; classification is heuristic.
pub fn analyze_regressions(
    repo: &Repository,
    max_commits: Option<usize>,
) -> anyhow::Result<RegressionReport> {
    let head = repo.head_commit_id()?;
    let limit = max_commits.unwrap_or(usize::MAX);

    // Per-file accumulation.
    let mut file_fixes: HashMap<PathBuf, Vec<i64>> = HashMap::new();
    let mut file_changes: HashMap<PathBuf, u32> = HashMap::new();
    let mut file_reverts: HashMap<PathBuf, u32> = HashMap::new();

    let mut reverts: Vec<RevertPair> = Vec::new();
    let mut total_fixes = 0u32;
    let mut total_reverts = 0u32;
    let mut total_commits = 0u32;

    for (i, id_res) in repo.rev_walk(head)?.enumerate() {
        if i >= limit {
            break;
        }
        let commit = repo.find_commit(id_res?)?;
        total_commits += 1;
        let classification = classify_commit_message(&commit.message);

        let parent_tree = match commit.parents.first() {
            Some(parent) => Some(repo.find_commit(*parent)?.tree_id),
            None => None,
        };
        let changes = repo.diff_tree_to_tree(parent_tree, commit.tree_id)?;

        let is_fix = classification == CommitClassification::Fix;
        let is_revert = classification == CommitClassification::Revert;

        if is_fix {
            total_fixes += 1;
        }
        if is_revert {
            total_reverts += 1;
        }

        // Track reverted oid from the message (e.g. "Revert \"...\"\n\nThis
        // reverts commit abc123...").
        let reverted_oid = extract_reverted_oid(&commit.message);

        for change in &changes {
            let path = change.path.clone();
            *file_changes.entry(path.clone()).or_insert(0) += 1;
            if is_fix {
                file_fixes
                    .entry(path.clone())
                    .or_default()
                    .push(commit.author.time);
            }
            if is_revert {
                *file_reverts.entry(path.clone()).or_insert(0) += 1;
            }
        }

        // A revert that touches the same files as a recent fix is a likely
        // regression signal — record it with the gap.
        if is_revert && !changes.is_empty() {
            let paths: Vec<PathBuf> = changes.iter().map(|c| c.path.clone()).collect();
            // Resolve the abbreviated oid from the message to compute the gap.
            let gap = reverted_oid.as_ref().and_then(|oid| {
                resolve_commit(repo, oid)
                    .ok()
                    .and_then(|id| repo.find_commit(id).ok())
                    .map(|rc| commit.author.time.saturating_sub(rc.author.time))
            });
            reverts.push(RevertPair {
                revert_oid: commit.id.to_string(),
                reverted_oid,
                paths,
                gap_seconds: gap,
            });
        }
    }

    // Build the problem-file list, sorted by fix density then fix count.
    let mut problem_files: Vec<ProblemFile> = file_fixes
        .into_iter()
        .map(|(path, fix_times)| {
            let total = *file_changes.get(&path).unwrap_or(&0);
            let reverts = *file_reverts.get(&path).unwrap_or(&0);
            let fix_density = if total > 0 {
                fix_times.len() as f64 / total as f64
            } else {
                0.0
            };
            ProblemFile {
                path,
                fix_commits: fix_times.len() as u32,
                total_changes: total,
                fix_density,
                reverts,
                fix_times,
            }
        })
        .collect();
    problem_files.sort_by(|a, b| {
        b.fix_density
            .partial_cmp(&a.fix_density)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| b.fix_commits.cmp(&a.fix_commits))
    });

    Ok(RegressionReport {
        problem_files,
        reverts,
        total_commits,
        total_fixes,
        total_reverts,
    })
}

/// Resolve a full or abbreviated hex oid to a commit, if it exists.
fn resolve_commit(repo: &Repository, oid: &str) -> anyhow::Result<gitx_git::models::ObjectId> {
    if let Some(full) = gitx_git::models::ObjectId::from_hex(oid) {
        if repo.object_kind(full)?.is_some() {
            return Ok(full);
        }
        anyhow::bail!("no such object");
    }
    let mut matches = Vec::new();
    for id_res in repo.all_object_ids()? {
        let id = id_res?;
        if id.to_string().starts_with(oid) {
            matches.push(id);
        }
    }
    match matches.len() {
        1 => Ok(matches[0]),
        _ => anyhow::bail!("ambiguous or missing object"),
    }
}

/// Extract a reverted commit oid from a "This reverts commit <oid>" line.
fn extract_reverted_oid(message: &str) -> Option<String> {
    for line in message.lines() {
        let lower = line.to_lowercase();
        if let Some(idx) = lower.find("reverts commit ") {
            let rest = &line[idx + "reverts commit ".len()..];
            let oid: String = rest.chars().take_while(|c| c.is_ascii_hexdigit()).collect();
            if oid.len() >= 7 {
                return Some(oid);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_reverted_oid() {
        let msg = "Revert \"feat: add x\"\n\nThis reverts commit a1b2c3d4e5f6a7b8.\n";
        assert_eq!(
            extract_reverted_oid(msg).as_deref(),
            Some("a1b2c3d4e5f6a7b8")
        );
        assert_eq!(extract_reverted_oid("no revert here"), None);
    }

    #[test]
    fn report_is_defaultable_and_serializable() {
        let report = RegressionReport::default();
        assert_eq!(report.total_commits, 0);
        let json = serde_json::to_string(&report).expect("serialize");
        assert!(json.contains("problem_files"));
    }

    #[test]
    fn classifies_fix_and_revert() {
        assert_eq!(
            classify_commit_message("fix: resolve crash"),
            CommitClassification::Fix
        );
        assert_eq!(
            classify_commit_message("Revert \"feat: x\""),
            CommitClassification::Revert
        );
    }
}
