//! Recovery analysis: reflog inspection and unreachable-commit detection.
//!
//! Everything here is read-only: no refs, objects or logs are ever modified.
//! The goal is to surface recoverable work (deleted branches, reset commits,
//! dangling commits) deterministically from repository data.

use gitx_git::models::ObjectId;
use gitx_git::repository::ObjectKind;
use gitx_git::{ReflogEntry, Repository};
use std::collections::HashSet;

/// A commit that exists in the object database but is not reachable from any
/// branch, tag or remote-tracking reference. Such commits are candidates for
/// garbage collection and are typically recoverable via the reflog.
#[derive(Debug, Clone)]
pub struct UnreachableCommit {
    pub oid: ObjectId,
}

/// The result of a recovery scan.
#[derive(Debug, Clone, Default)]
pub struct RecoveryReport {
    /// Reflog entries across all local references (newest first per ref).
    pub reflog: Vec<ReflogEntry>,
    /// Commits not reachable from any ref.
    pub unreachable: Vec<UnreachableCommit>,
    /// Whether the repository has reflogs at all.
    pub reflog_enabled: bool,
}

/// Collect the reflog of every local reference (`HEAD`, branches, tags),
/// newest entry first.
pub fn collect_reflog(repo: &Repository) -> gitx_git::Result<Vec<ReflogEntry>> {
    let mut entries = repo.reflog("HEAD")?;
    for branch in repo.branches()? {
        let name = if branch.is_remote {
            format!("refs/remotes/{}", branch.name)
        } else {
            format!("refs/heads/{}", branch.name)
        };
        entries.extend(repo.reflog(&name)?);
    }
    entries.sort_by_key(|e| std::cmp::Reverse(e.timestamp));
    Ok(entries)
}

/// Find commits present in the object database but not reachable from any
/// branch, tag or remote-tracking reference.
///
/// `max_objects` caps the object-database scan (docs/12 performance
/// guidance); pass `None` to scan everything.
pub fn find_unreachable_commits(
    repo: &Repository,
    max_objects: Option<usize>,
) -> gitx_git::Result<Vec<UnreachableCommit>> {
    let mut reachable: HashSet<String> = HashSet::new();

    for branch in repo.branches()? {
        collect_reachable(repo, branch.target, &mut reachable)?;
    }
    for tag in repo.tags()? {
        collect_reachable(repo, tag.target, &mut reachable)?;
    }

    let mut unreachable = Vec::new();
    for (idx, id_res) in repo.all_object_ids()?.enumerate() {
        if let Some(max) = max_objects
            && idx >= max
        {
            break;
        }
        let id = id_res?;
        if !reachable.contains(&id.to_string()) && repo.object_kind(id)? == Some(ObjectKind::Commit)
        {
            unreachable.push(UnreachableCommit { oid: id });
        }
    }
    Ok(unreachable)
}

fn collect_reachable(
    repo: &Repository,
    tip: ObjectId,
    out: &mut HashSet<String>,
) -> gitx_git::Result<()> {
    for id_res in repo.rev_walk(tip)? {
        out.insert(id_res?.to_string());
    }
    Ok(())
}

/// Build a full recovery report.
pub fn analyze_recovery(repo: &Repository) -> gitx_git::Result<RecoveryReport> {
    let reflog = collect_reflog(repo)?;
    let unreachable = find_unreachable_commits(repo, None)?;
    Ok(RecoveryReport {
        reflog_enabled: !reflog.is_empty(),
        reflog,
        unreachable,
    })
}

/// Recovery report with a capped object-database scan — used by the TUI so
/// startup stays fast on large repositories (docs/12 performance guidance).
/// The CLI `gitx recovery` uses the full [`analyze_recovery`] instead.
pub fn analyze_recovery_capped(repo: &Repository) -> gitx_git::Result<RecoveryReport> {
    let reflog = collect_reflog(repo)?;
    let unreachable = find_unreachable_commits(repo, Some(10_000))?;
    Ok(RecoveryReport {
        reflog_enabled: !reflog.is_empty(),
        reflog,
        unreachable,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unreachable_report_is_defaultable() {
        let report = RecoveryReport::default();
        assert!(report.reflog.is_empty());
        assert!(report.unreachable.is_empty());
    }
}
