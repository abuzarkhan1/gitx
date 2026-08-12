use crate::repository::Repository;
use gitx_index::contracts::GitProvider;
use gitx_index::models::{Commit, Oid, RefInfo};
use gitx_index::IndexerError;
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
                name: format!("refs/tags/{}", tag.name),
                target: Oid(tag.target.to_string()),
            });
        }

        Ok(refs)
    }

    fn walk_commits(
        &self,
        starting_from: &[Oid],
    ) -> Result<Box<dyn Iterator<Item = Result<Commit, IndexerError>> + '_>, IndexerError> {
        // Chain the rev-walks of every starting point; the engine dedupes
        // visited commits and stops at already-indexed boundaries.
        let mut iterators: Vec<
            Box<dyn Iterator<Item = Result<Commit, gitx_index::IndexerError>> + '_>,
        > = Vec::new();

        for start in starting_from {
            let head = crate::models::ObjectId::from_hex(&start.0).ok_or_else(|| {
                gitx_index::IndexerError::GitError(format!("invalid oid {}", start.0))
            })?;
            let walk = self
                .rev_walk(head)
                .map_err(|e| gitx_index::IndexerError::GitError(e.to_string()))?;
            let mapped = walk.map(|res| {
                res.map(|id| {
                    let commit = self.find_commit(id).ok();
                    Commit {
                        id: Oid(id.to_string()),
                        parents: commit
                            .as_ref()
                            .map(|c| c.parents.iter().map(|p| Oid(p.to_string())).collect())
                            .unwrap_or_default(),
                        message: commit.as_ref().map(|c| c.message.clone()),
                        timestamp: commit.as_ref().map(|c| c.author.time),
                        author_name: commit.as_ref().map(|c| c.author.name.clone()),
                        author_email: commit.as_ref().map(|c| c.author.email.clone()),
                        committer_name: commit.as_ref().map(|c| c.committer.name.clone()),
                        committer_email: commit.as_ref().map(|c| c.committer.email.clone()),
                        tree_id: commit.as_ref().map(|c| c.tree_id.to_string()),
                    }
                })
                .map_err(|e| gitx_index::IndexerError::GitError(e.to_string()))
            });
            iterators.push(Box::new(mapped));
        }

        Ok(Box::new(iterators.into_iter().flatten()))
    }
}
