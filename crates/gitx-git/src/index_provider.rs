use crate::models::ObjectId;
use crate::repository::Repository;
use gitx_index::contracts::GitProvider;
use gitx_index::models::{Commit, Oid, RefInfo};
use gitx_index::IndexerError;
use std::collections::{HashSet, VecDeque};
use std::result::Result;

/// Adapts [`gitx_git::Repository`] to the [`GitProvider`] contract used by the
/// incremental indexer (docs/09).
impl GitProvider for Repository {
    fn head_ref_name(&self) -> Result<Option<String>, IndexerError> {
        self.head_ref_name()
            .map_err(|e| IndexerError::GitError(e.to_string()))
    }

    fn read_refs(&self) -> Result<Vec<RefInfo>, IndexerError> {
        let mut refs = Vec::new();

        for branch in self
            .branches()
            .map_err(|e| gitx_index::IndexerError::GitError(e.to_string()))?
        {
            let name = if branch.is_remote {
                format!("refs/remotes/{}", branch.name)
            } else {
                format!("refs/heads/{}", branch.name)
            };
            refs.push(RefInfo {
                name,
                target: Oid(branch.target.to_string()),
            });
        }

        for tag in self
            .tags()
            .map_err(|e| gitx_index::IndexerError::GitError(e.to_string()))?
        {
            refs.push(RefInfo {
                name: format!("refs/tags/{tag}", tag = tag.name),
                target: Oid(tag.target.to_string()),
            });
        }

        Ok(refs)
    }

    fn walk_commits<'s, 'i, 'w>(
        &'s self,
        starting_from: &[Oid],
        indexed: &'i HashSet<String>,
    ) -> Result<Box<dyn Iterator<Item = Result<Commit, IndexerError>> + 'w>, IndexerError>
    where
        's: 'w,
        'i: 'w,
    {
        // Seed the queue with every tip that is not already indexed. An
        // indexed tip's ancestry is by construction already indexed, so it is
        // never read from the object database at all.
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        for start in starting_from {
            let head = ObjectId::from_hex(&start.0).ok_or_else(|| {
                gitx_index::IndexerError::GitError(format!("invalid oid {}", start.0))
            })?;
            if indexed.contains(&start.0) {
                continue;
            }
            if visited.insert(head) {
                queue.push_back(head);
            }
        }
        Ok(Box::new(BoundaryWalk {
            repo: self,
            indexed,
            queue,
            visited,
        }))
    }
}

/// A walk that stops descending at already-indexed commits (docs/13 §3
/// incremental refresh).
///
/// Only commits NOT in `indexed` are visited — and they are fully decoded,
/// since they are exactly the commits the index needs. A commit that is
/// already indexed is never read from the object database, so an incremental
/// refresh touches O(new commits) objects instead of re-walking (and
/// re-decompressing) the entire history of a large repository. This is what
/// keeps `[index] auto_refresh` sub-second when the repo has moved by a few
/// commits.
struct BoundaryWalk<'a> {
    repo: &'a Repository,
    indexed: &'a HashSet<String>,
    queue: VecDeque<ObjectId>,
    /// Commits already scheduled or visited (only unindexed ones — indexed
    /// commits are filtered before enqueue), so a commit reachable from
    /// several refs/parents is decoded exactly once.
    visited: HashSet<ObjectId>,
}

impl Iterator for BoundaryWalk<'_> {
    type Item = Result<Commit, IndexerError>;

    fn next(&mut self) -> Option<Self::Item> {
        let id = self.queue.pop_front()?;
        let commit = match self.repo.find_commit(id) {
            Ok(c) => c,
            Err(e) => return Some(Err(gitx_index::IndexerError::GitError(e.to_string()))),
        };
        for parent in &commit.parents {
            let key = parent.to_string();
            if !self.indexed.contains(&key) && !self.visited.contains(parent) {
                self.visited.insert(*parent);
                self.queue.push_back(*parent);
            }
        }
        Some(Ok(Commit {
            id: Oid(commit.id.to_string()),
            parents: commit.parents.iter().map(|p| Oid(p.to_string())).collect(),
            message: Some(commit.message),
            timestamp: Some(commit.author.time),
            author_name: Some(commit.author.name),
            author_email: Some(commit.author.email),
            committer_name: Some(commit.committer.name),
            committer_email: Some(commit.committer.email),
            tree_id: Some(commit.tree_id.to_string()),
        }))
    }
}
