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

/// A non-commit object (tree or blob) present in the object database but not
/// reachable from any reference. Dangling blobs are typically the payload of
/// `git add`-then-`git reset` accidents; dangling trees usually accompany
/// unreachable commits (docs/12 §6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DanglingKind {
    Tree,
    Blob,
}

impl std::fmt::Display for DanglingKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DanglingKind::Tree => write!(f, "tree"),
            DanglingKind::Blob => write!(f, "blob"),
        }
    }
}

/// A dangling tree or blob object.
#[derive(Debug, Clone)]
pub struct DanglingObject {
    pub oid: ObjectId,
    pub kind: DanglingKind,
}

/// The result of a recovery scan.
#[derive(Debug, Clone, Default)]
pub struct RecoveryReport {
    /// Reflog entries across all local references (newest first per ref).
    pub reflog: Vec<ReflogEntry>,
    /// Commits not reachable from any ref.
    pub unreachable: Vec<UnreachableCommit>,
    /// Dangling trees/blobs not reachable from any ref (bounded scan; docs/12 §6).
    pub dangling: Vec<DanglingObject>,
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

/// Find dangling trees and blobs: objects of those kinds present in the object
/// database but not reachable from any reference's commit graph.
///
/// Reachability is computed from the reachable commit set (rev-walks of every
/// branch/tag) and their trees. To stay bounded on large repositories the tree
/// walk is capped at `max_trees` reachable commit trees (newest commits first);
/// pass `None` to walk every reachable commit's tree.
pub fn find_dangling_objects(
    repo: &Repository,
    max_trees: Option<usize>,
    max_objects: Option<usize>,
) -> gitx_git::Result<Vec<DanglingObject>> {
    let mut reachable: HashSet<String> = HashSet::new();

    // Commit reachability + the trees/blobs they contain.
    let mut commits = Vec::new();
    for branch in repo.branches()? {
        commits.extend(repo.rev_walk(branch.target)?);
    }
    for tag in repo.tags()? {
        commits.extend(repo.rev_walk(tag.target)?);
    }
    let mut walked = 0usize;
    for id_res in commits {
        let id = id_res?;
        let oid = id.to_string();
        if reachable.insert(oid.clone()) {
            if let Some(max) = max_trees
                && walked >= max
            {
                continue;
            }
            if let Ok(commit) = repo.find_commit(id) {
                walked += 1;
                if let Ok(trees) = repo.tree_oids(commit.tree_id) {
                    for t in trees {
                        reachable.insert(t.to_string());
                    }
                }
                if let Ok(entries) = repo.tree_entries(commit.tree_id) {
                    for (_, blob) in entries {
                        reachable.insert(blob.to_string());
                    }
                }
            }
        }
    }

    let mut dangling = Vec::new();
    for (idx, id_res) in repo.all_object_ids()?.enumerate() {
        if let Some(max) = max_objects
            && idx >= max
        {
            break;
        }
        let id = id_res?;
        let oid = id.to_string();
        if reachable.contains(&oid) {
            continue;
        }
        match repo.object_kind(id)? {
            Some(ObjectKind::Tree) => dangling.push(DanglingObject {
                oid: id,
                kind: DanglingKind::Tree,
            }),
            Some(ObjectKind::Blob) => dangling.push(DanglingObject {
                oid: id,
                kind: DanglingKind::Blob,
            }),
            _ => {}
        }
    }
    Ok(dangling)
}

/// Build a full recovery report.
pub fn analyze_recovery(repo: &Repository) -> gitx_git::Result<RecoveryReport> {
    tracing::info!("recovery scan start");
    let reflog = collect_reflog(repo)?;
    let unreachable = find_unreachable_commits(repo, None)?;
    let dangling = find_dangling_objects(repo, None, None)?;
    tracing::info!(
        reflog_entries = reflog.len(),
        unreachable_commits = unreachable.len(),
        dangling_objects = dangling.len(),
        "recovery scan complete"
    );
    Ok(RecoveryReport {
        reflog_enabled: !reflog.is_empty(),
        reflog,
        unreachable,
        dangling,
    })
}

/// Recovery report with a capped object-database scan — used by the TUI so
/// startup stays fast on large repositories (docs/12 performance guidance).
/// The CLI `gitx recovery` uses the full [`analyze_recovery`] instead.
pub fn analyze_recovery_capped(repo: &Repository) -> gitx_git::Result<RecoveryReport> {
    let reflog = collect_reflog(repo)?;
    let unreachable = find_unreachable_commits(repo, Some(10_000))?;
    let dangling = find_dangling_objects(repo, Some(200), Some(10_000))?;
    Ok(RecoveryReport {
        reflog_enabled: !reflog.is_empty(),
        reflog,
        unreachable,
        dangling,
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
