use chrono::{DateTime, Utc};
use gitx_core::id::CommitId;

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
