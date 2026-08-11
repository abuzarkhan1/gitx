use chrono::{DateTime, Utc};
use gitx_core::id::CommitId;
use gitx_git::Repository;
use gitx_git::models::{Branch, ObjectId};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/// Branch intelligence data
#[derive(Debug, Clone)]
pub struct BranchIntelligence {
    pub ahead: u32,
    pub behind: u32,
    pub merge_base: Option<CommitId>,
    pub diverged_commits: u32,
    pub branch_age_days: i64,
    pub recent_activity_days: i64,
    pub shared_files: u32,
    pub is_stale: bool,
    /// Merge complexity is an estimate and must be labeled as such.
    pub merge_complexity: f64,
}

/// Analyze a branch based on its metrics.
#[allow(clippy::too_many_arguments)]
pub fn analyze_branch(
    ahead: u32,
    behind: u32,
    merge_base: Option<CommitId>,
    created_at: DateTime<Utc>,
    last_commit_at: DateTime<Utc>,
    now: DateTime<Utc>,
    shared_files_changed: u32,
    overlapping_directories: u32,
) -> BranchIntelligence {
    let branch_age_days = (now - created_at).num_days();
    let recent_activity_days = (now - last_commit_at).num_days();

    // Heuristic: Stale if older than 30 days and no activity in 14 days
    let is_stale = branch_age_days > 30 && recent_activity_days > 14;

    let diverged_commits = ahead + behind;

    // Merge complexity = weighted overlap of changed files + number of diverged commits + overlapping directories
    let merge_complexity = (shared_files_changed as f64 * 2.0)
        + (diverged_commits as f64)
        + (overlapping_directories as f64 * 1.5);

    BranchIntelligence {
        ahead,
        behind,
        merge_base,
        diverged_commits,
        branch_age_days,
        recent_activity_days,
        shared_files: shared_files_changed,
        is_stale,
        merge_complexity,
    }
}

/// Commits reachable from `tip`, as a set of abbreviated oids.
fn reachable_set(repo: &Repository, tip: ObjectId) -> anyhow::Result<HashSet<String>> {
    let mut set = HashSet::new();
    for id_res in repo.rev_walk(tip)? {
        set.insert(id_res?.to_string());
    }
    Ok(set)
}

/// Files changed by the commits in `oids` (bounded to keep the estimate fast
/// on large repositories).
fn changed_files(repo: &Repository, oids: &HashSet<String>, cap: usize) -> HashSet<PathBuf> {
    let mut out = HashSet::new();
    for id_str in oids.iter().take(cap) {
        let Some(id) = ObjectId::from_hex(id_str) else {
            continue;
        };
        let Ok(commit) = repo.find_commit(id) else {
            continue;
        };
        let parent_tree = match commit.parents.first() {
            Some(parent) => repo.find_commit(*parent).ok().map(|p| p.tree_id),
            None => None,
        };
        if let Ok(changes) = repo.diff_tree_to_tree(parent_tree, commit.tree_id) {
            for change in changes {
                out.insert(change.path);
            }
        }
    }
    out
}

/// Compute full branch intelligence for `branch` against `base` from actual
/// Git ancestry (docs/10 §5): ahead/behind, divergence, age, activity, shared
/// files, staleness, and the merge-complexity estimate (which is labeled an
/// estimate, never a conflict guarantee).
pub fn branch_intelligence(
    repo: &Repository,
    branch: &Branch,
    base: Option<&Branch>,
) -> anyhow::Result<Option<BranchIntelligence>> {
    let Some(base) = base.filter(|b| b.name != branch.name) else {
        // A branch compared with itself has no divergence.
        let tip = repo.find_commit(branch.target)?;
        let now = Utc::now();
        let tip_dt = DateTime::from_timestamp(tip.author.time, 0).unwrap_or(now);
        return Ok(Some(analyze_branch(0, 0, None, tip_dt, tip_dt, now, 0, 0)));
    };

    let ours = reachable_set(repo, branch.target)?;
    let theirs = reachable_set(repo, base.target)?;
    let ahead_set: HashSet<String> = ours.difference(&theirs).cloned().collect();
    let behind_set: HashSet<String> = theirs.difference(&ours).cloned().collect();
    let ahead = ahead_set.len() as u32;
    let behind = behind_set.len() as u32;

    // Shared files: changed on both sides since divergence (docs/10 §5,
    // docs/24 §8). Bounded to the first 200 diverged commits each way.
    let ours_files = changed_files(repo, &ahead_set, 200);
    let theirs_files = changed_files(repo, &behind_set, 200);
    let shared_files = ours_files.intersection(&theirs_files).count() as u32;

    // Age = days since the oldest diverged commit (branch creation proxy);
    // activity = days since the tip.
    let now = Utc::now();
    let tip = repo.find_commit(branch.target)?;
    let tip_dt = DateTime::from_timestamp(tip.author.time, 0).unwrap_or(now);

    let created_dt = if ahead_set.is_empty() {
        tip_dt
    } else {
        let mut oldest: Option<DateTime<Utc>> = None;
        for id_str in ahead_set.iter().take(200) {
            let Some(id) = ObjectId::from_hex(id_str) else {
                continue;
            };
            if let Ok(c) = repo.find_commit(id) {
                let dt = DateTime::from_timestamp(c.author.time, 0).unwrap_or(now);
                oldest = Some(match oldest {
                    Some(o) if o <= dt => o,
                    _ => dt,
                });
            }
        }
        oldest.unwrap_or(tip_dt)
    };

    // Per-directory overlap (docs/10 §5 merge-complexity term).
    let dir_overlap = {
        let mut ours_dirs: HashMap<String, u32> = HashMap::new();
        let mut theirs_dirs: HashMap<String, u32> = HashMap::new();
        for p in &ours_files {
            if let Some(dir) = p.parent() {
                *ours_dirs.entry(dir.display().to_string()).or_insert(0) += 1;
            }
        }
        for p in &theirs_files {
            if let Some(dir) = p.parent() {
                *theirs_dirs.entry(dir.display().to_string()).or_insert(0) += 1;
            }
        }
        let mut overlap = 0;
        for (dir, count) in &ours_dirs {
            if let Some(other) = theirs_dirs.get(dir) {
                overlap += count.min(other);
            }
        }
        overlap
    };

    Ok(Some(analyze_branch(
        ahead,
        behind,
        None,
        created_dt,
        tip_dt,
        now,
        shared_files,
        dir_overlap,
    )))
}

/// Aggregate ownership per directory/module (docs/10 §4 subsystem ownership).
/// Returns (directory, total lines, top contributor, concentration %).
pub fn subsystem_ownership(
    author_lines: &HashMap<PathBuf, HashMap<String, u64>>,
) -> Vec<(String, u64, String, f64)> {
    let mut dirs: HashMap<String, (u64, HashMap<String, u64>)> = HashMap::new();
    for (path, authors) in author_lines {
        let dir = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "(root)".to_string());
        let entry = dirs.entry(dir).or_default();
        for (author, lines) in authors {
            *entry.1.entry(author.clone()).or_insert(0) += lines;
            entry.0 += lines;
        }
    }
    let mut out: Vec<(String, u64, String, f64)> = dirs
        .into_iter()
        .map(|(dir, (total, authors))| {
            let top = authors.iter().max_by_key(|(_, l)| **l);
            let (top_name, top_lines) = top.map(|(n, l)| (n.clone(), *l)).unwrap_or_default();
            let concentration = if total == 0 {
                0.0
            } else {
                top_lines as f64 / total as f64 * 100.0
            };
            (dir, total, top_name, concentration)
        })
        .collect();
    out.sort_by_key(|(_, total, _, _)| std::cmp::Reverse(*total));
    out
}
