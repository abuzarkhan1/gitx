use chrono::{TimeZone, Utc};
use gitx_git::Repository;
use gitx_git::models::{Commit, ObjectId};
use gitx_git::reflog::split_lines;
use rayon::ThreadPool;
use rayon::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::Instant;

use crate::classification::classify_commit_message;
use crate::health::RepoHealth;
use crate::hotspots::{HotspotWeights, calculate_hotspot_score_with, calculate_risk_score};
use crate::metrics::FileMetrics;
use gitx_core::types::CommitClassification;

/// Signals are computed over this trailing window for "recent" metrics.
pub const RECENT_DAYS: i64 = 30;

/// Per-file analysis produced by the pipeline.
#[derive(Debug, Clone)]
pub struct FileAnalysis {
    pub path: PathBuf,
    pub metrics: FileMetrics,
    /// Author key (`Name <email>`) → lines they added to this file.
    pub author_lines: HashMap<String, u64>,
    /// Share of the file owned by its top contributor, 0–100.
    pub ownership_concentration: f64,
    /// Change/maintenance risk hotspot score, 0–100.
    pub hotspot: f64,
    /// LOW / MEDIUM / HIGH / CRITICAL band.
    pub classification: &'static str,
    /// Composite risk score (evidence-backed, see docs/10).
    pub risk: f64,
}

/// Bounded worker pool for the analysis walk (docs/13 §6): CPU-heavy,
/// independent work (commit diffs, blob reads) is parallelized without
/// uncontrolled threads. Capped at 8 workers to keep memory bounded on
/// large repositories.
pub fn analysis_pool() -> &'static ThreadPool {
    static POOL: OnceLock<ThreadPool> = OnceLock::new();
    POOL.get_or_init(|| {
        let workers = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1)
            .clamp(1, 8);
        rayon::ThreadPoolBuilder::new()
            .num_threads(workers)
            .thread_name(|i| format!("gitx-analysis-{i}"))
            .build()
            .expect("failed to build analysis thread pool")
    })
}

/// A commit plus the (path, insertions, deletions) deltas it introduced,
/// extracted in parallel from the commit graph.
struct CommitDeltas {
    commit: Commit,
    changes: Vec<(PathBuf, u32, u32)>,
}

/// Repository-wide analysis result.
#[derive(Debug, Clone)]
pub struct RepoAnalysis {
    pub files: Vec<FileAnalysis>,
    pub total_commits: u64,
    pub total_contributors: usize,
    pub current_files: usize,
    pub health: RepoHealth,
    pub analysis_duration_ms: u128,
}

#[derive(Default)]
struct FileAcc {
    change_frequency: u32,
    lines_added: u64,
    lines_deleted: u64,
    recent_churn: u64,
    recent_changes: u32,
    bug_fix_count: u32,
    first_introduced: Option<i64>,
    last_modified: Option<i64>,
    authors: HashMap<String, u64>,
}

/// Walk the repository history (mainline from HEAD) and compute per-file
/// metrics, hotspots, risk, ownership and the composite health score.
///
/// This is the deterministic "Repository Intelligence" pipeline from
/// docs/10 + docs/24: every number is derived from Git data, never guessed.
pub fn analyze_repository(repo: &Repository) -> anyhow::Result<RepoAnalysis> {
    analyze_repository_with(repo, HotspotWeights::default())
}

/// Like [`analyze_repository`], with caller-provided hotspot weights
/// (configurable via `[analysis]`, docs/16 §3).
pub fn analyze_repository_with(
    repo: &Repository,
    weights: HotspotWeights,
) -> anyhow::Result<RepoAnalysis> {
    let started = Instant::now();
    let now = Utc::now().timestamp();
    let cutoff = now - RECENT_DAYS * 86_400;
    tracing::debug!("repository analysis start");

    let head_id = repo.head_commit_id()?;
    let head_commit = repo.find_commit(head_id)?;

    // 1. Collect the commit list (deterministic order: newest first).
    let mut commit_ids: Vec<ObjectId> = Vec::new();
    for commit_id_res in repo.rev_walk(head_id)? {
        commit_ids.push(commit_id_res?);
    }

    // 2. Parallel walk (docs/13 §6): diff computation per commit is
    //    independent, CPU-heavy work, so it runs on the bounded pool. The
    //    results are folded back in original order, making the accumulated
    //    metrics bit-for-bit identical to a sequential walk. The gix handle
    //    is Send but not Sync, so each chunk opens its own handle against the
    //    same git dir (the supported parallel-read pattern).
    let workers = analysis_pool().current_num_threads();
    let git_dir = repo.git_dir().to_path_buf();
    let chunk = commit_ids.len().div_ceil(workers).max(1);
    let per_commit: Vec<CommitDeltas> = analysis_pool().install(|| {
        commit_ids
            .par_chunks(chunk)
            .map(|chunk| {
                let local = Repository::open(&git_dir)?;
                let mut out = Vec::with_capacity(chunk.len());
                for cid in chunk {
                    let commit = local.find_commit(*cid)?;
                    let parent_tree = match commit.parents.first() {
                        Some(parent) => Some(local.find_commit(*parent)?.tree_id),
                        None => None,
                    };
                    let changes = local.diff_tree_to_tree(parent_tree, commit.tree_id)?;
                    out.push(CommitDeltas {
                        commit,
                        changes: changes
                            .iter()
                            .map(|c| (c.path.clone(), c.insertions, c.deletions))
                            .collect(),
                    });
                }
                Ok(out)
            })
            .collect::<anyhow::Result<Vec<Vec<CommitDeltas>>>>()
            .map(|chunks| chunks.into_iter().flatten().collect::<Vec<CommitDeltas>>())
    })?;

    // 3. Sequential fold — same order as a linear walk, so results are
    //    deterministic regardless of pool size.
    let mut accs: HashMap<PathBuf, FileAcc> = HashMap::new();
    let mut total_commits = 0u64;
    let mut total_changes = 0u64;
    let mut recent_changes = 0u64;
    let mut author_set: std::collections::HashSet<String> = std::collections::HashSet::new();

    for CommitDeltas { commit, changes } in per_commit {
        total_commits += 1;

        let author_key = author_key(&commit);
        author_set.insert(author_key.clone());
        let is_fix = classify_commit_message(&commit.message) == CommitClassification::Fix;
        let is_recent = commit.author.time >= cutoff;

        for (path, insertions, deletions) in changes {
            total_changes += 1;
            let acc = accs.entry(path).or_default();

            acc.change_frequency += 1;
            acc.lines_added += insertions as u64;
            acc.lines_deleted += deletions as u64;
            if is_recent {
                acc.recent_churn += (insertions + deletions) as u64;
                acc.recent_changes += 1;
                recent_changes += 1;
            }
            if is_fix {
                acc.bug_fix_count += 1;
            }
            if acc.first_introduced.is_none() {
                acc.first_introduced = Some(commit.author.time);
            }
            acc.last_modified = Some(commit.author.time);
            *acc.authors.entry(author_key.clone()).or_insert(0) += insertions as u64;
        }
    }

    // 4. Current (HEAD) file list — for complexity and stability signals.
    let current_paths = repo.list_blobs(head_commit.tree_id)?;
    let current_set: std::collections::HashSet<PathBuf> = current_paths.iter().cloned().collect();

    // 5. Line counts for every analyzed file (parallel blob reads, docs/13 §6).
    let paths: Vec<PathBuf> = accs.keys().cloned().collect();
    let chunk = paths.len().div_ceil(workers).max(1);
    let locs: HashMap<PathBuf, u32> = analysis_pool().install(|| {
        paths
            .par_chunks(chunk)
            .map(|chunk| {
                let local = Repository::open(&git_dir)?;
                let mut out = Vec::with_capacity(chunk.len());
                for path in chunk {
                    let loc = local
                        .blob_at_path(head_commit.tree_id, path)
                        .ok()
                        .flatten()
                        .map(|b| split_lines(&b).len() as u32)
                        .unwrap_or(0);
                    out.push((path.clone(), loc));
                }
                Ok(out)
            })
            .collect::<anyhow::Result<Vec<Vec<(PathBuf, u32)>>>>()
            .map(|chunks| {
                chunks
                    .into_iter()
                    .flatten()
                    .collect::<HashMap<PathBuf, u32>>()
            })
    })?;

    // 6. Normalize each signal to 0–100 across files (max-based scaling).
    let n = accs.len();
    let max_frequency = accs.values().map(|a| a.change_frequency).max().unwrap_or(0);
    let max_churn = accs.values().map(|a| a.recent_churn).max().unwrap_or(0);
    let max_bugs = accs.values().map(|a| a.bug_fix_count).max().unwrap_or(0);
    let max_complexity = locs.values().copied().max().unwrap_or(0);

    let mut files = Vec::with_capacity(n);
    let mut recently_added = 0usize;

    for (path, acc) in &accs {
        // Current line count of the file (complexity proxy, docs/10 §2),
        // read in parallel during step 5.
        let loc = locs.get(path).copied().unwrap_or(0);

        let n_frequency = scale(acc.change_frequency, max_frequency);
        let n_churn = scale(acc.recent_churn as u32, max_churn as u32);
        let n_bugs = scale(acc.bug_fix_count, max_bugs);
        let n_complexity = scale(loc, max_complexity);

        let ownership = ownership_concentration(&acc.authors);
        let hotspot = calculate_hotspot_score_with(
            weights,
            n_frequency,
            n_churn,
            n_bugs,
            ownership,
            n_complexity,
            RECENT_DAYS as u32,
        );
        // Docs/10 §6 risk is the sum of four 0–100 components; normalize to a
        // 0–100 composite (matching the `/100` presentation in the PRD).
        let risk = calculate_risk_score(&hotspot, ownership, n_churn, n_complexity) / 4.0;

        let first = acc
            .first_introduced
            .and_then(|t| Utc.timestamp_opt(t, 0).single());

        files.push(FileAnalysis {
            path: path.clone(),
            metrics: FileMetrics {
                change_frequency: acc.change_frequency,
                lines_added: acc.lines_added as u32,
                lines_deleted: acc.lines_deleted as u32,
                first_introduced: first,
                last_modified: acc
                    .last_modified
                    .and_then(|t| Utc.timestamp_opt(t, 0).single()),
                bug_fix_count: acc.bug_fix_count,
                unique_contributors: acc.authors.len() as u32,
            },
            author_lines: acc.authors.clone(),
            ownership_concentration: ownership,
            hotspot: hotspot.value,
            classification: hotspot.classification,
            risk,
        });

        if let Some(first) = first
            && current_set.contains(path)
            && first.timestamp() >= cutoff
        {
            recently_added += 1;
        }
    }

    files.sort_by(|a, b| {
        b.hotspot
            .partial_cmp(&a.hotspot)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // 4. Composite health (docs/10 §8, docs/01 goal G2).
    let health = compute_health(
        repo,
        &files,
        total_changes,
        recent_changes,
        recently_added,
        current_set.len(),
    )?;

    tracing::info!(
        commits = total_commits,
        files = files.len(),
        workers,
        elapsed_ms = started.elapsed().as_millis() as u64,
        "repository analysis complete"
    );
    Ok(RepoAnalysis {
        total_commits,
        total_contributors: author_set.len(),
        current_files: current_set.len(),
        files,
        health,
        analysis_duration_ms: started.elapsed().as_millis(),
    })
}

fn compute_health(
    repo: &Repository,
    files: &[FileAnalysis],
    total_changes: u64,
    recent_changes: u64,
    recently_added: usize,
    current_files: usize,
) -> anyhow::Result<RepoHealth> {
    let total = files.len().max(1);

    // Code hotspots: fewer high-risk files is healthier.
    let high_risk = files
        .iter()
        .filter(|f| f.classification == "HIGH" || f.classification == "CRITICAL")
        .count();
    let code_hotspots_score = 100.0 - (high_risk as f64 / total as f64) * 100.0;

    // Ownership risk: higher average concentration is riskier (bus factor).
    let ownership_risk_score =
        files.iter().map(|f| f.ownership_concentration).sum::<f64>() / total as f64;

    // Branch hygiene: share of branches with activity in the last 30 days.
    let branches = repo.branches()?;
    let cutoff = Utc::now().timestamp() - RECENT_DAYS * 86_400;
    let active = branches
        .iter()
        .filter(|b| {
            repo.find_commit(b.target)
                .map(|c| c.author.time >= cutoff)
                .unwrap_or(false)
        })
        .count();
    let branch_hygiene_score = if branches.is_empty() {
        100.0
    } else {
        active as f64 / branches.len() as f64 * 100.0
    };

    // Change volatility: share of changes in the recent window.
    let change_volatility_score = if total_changes == 0 {
        0.0
    } else {
        recent_changes as f64 / total_changes as f64 * 100.0
    };

    // Architecture stability: share of current files that were NOT added recently.
    let architecture_stability_score = if current_files == 0 {
        100.0
    } else {
        (1.0 - recently_added as f64 / current_files as f64) * 100.0
    };

    // Recovery risk: no reflog → nothing recoverable; dangling commits raise risk.
    let reflog = repo.head_reflog()?;
    let unreachable = crate::recovery::find_unreachable_commits(repo, Some(10_000))?;
    let recovery_risk_score = if reflog.is_empty() {
        100.0
    } else {
        (unreachable.len() as f64 * 10.0).min(100.0)
    };

    Ok(RepoHealth::calculate(
        code_hotspots_score,
        ownership_risk_score,
        branch_hygiene_score,
        change_volatility_score,
        architecture_stability_score,
        recovery_risk_score,
    ))
}

fn author_key(commit: &Commit) -> String {
    format!("{} <{}>", commit.author.name, commit.author.email)
}

/// Share (0–100) of the file contributed by its top author.
fn ownership_concentration(authors: &HashMap<String, u64>) -> f64 {
    let total: u64 = authors.values().sum();
    if total == 0 {
        return 0.0;
    }
    let max = authors.values().max().copied().unwrap_or(0);
    max as f64 / total as f64 * 100.0
}

/// Max-based normalization to 0–100.
fn scale(value: u32, max: u32) -> f64 {
    if max == 0 {
        0.0
    } else {
        value as f64 / max as f64 * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn ownership_concentration_is_percentage() {
        let mut authors = HashMap::new();
        authors.insert("a".to_string(), 9);
        authors.insert("b".to_string(), 1);
        assert_eq!(ownership_concentration(&authors), 90.0);
    }

    #[test]
    fn ownership_concentration_empty() {
        assert_eq!(ownership_concentration(&HashMap::new()), 0.0);
    }

    #[test]
    fn scale_normalizes_by_max() {
        assert_eq!(scale(0, 10), 0.0);
        assert_eq!(scale(5, 10), 50.0);
        assert_eq!(scale(10, 10), 100.0);
        assert_eq!(scale(42, 0), 0.0);
    }
}
