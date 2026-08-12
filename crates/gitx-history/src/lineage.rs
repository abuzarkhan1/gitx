use crate::timeline::HistoryService;
use gitx_git::Repository;
use gitx_git::models::ObjectId;
use std::path::{Path, PathBuf};

/// One step in a file's life, newest first.
#[derive(Debug, Clone)]
pub struct FileLineageNode {
    pub commit_id: ObjectId,
    /// The path the file had at this commit (i.e. after the commit's change,
    /// going forward in time).
    pub path: PathBuf,
    pub action: FileAction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileAction {
    /// The file appeared. `copy_of` is set when the added blob is identical
    /// to an existing file's blob in the same tree (docs/02 §2 “copies where
    /// reliably detectable”).
    Added {
        copy_of: Option<PathBuf>,
    },
    Modified,
    Deleted,
    /// The file was renamed `from → path` by this commit.
    Renamed {
        from: PathBuf,
    },
}

/// The full life of a file: every commit that touched it, following renames
/// backward from HEAD (docs/10 file archaeology, docs/02 §2).
#[derive(Debug, Clone)]
pub struct FileLineage {
    pub history: Vec<FileLineageNode>,
}

impl HistoryService<'_> {
    /// Walk the mainline backward from `from` (default HEAD), following the
    /// file's path across renames. For each commit that touched the file —
    /// added, modified, deleted, or renamed it — a node is recorded, newest
    /// first. Rename detection uses the gix tree diff's `old_path` (the same
    /// signal `gitx history`/blame use), not name similarity.
    pub fn get_file_lineage(
        &self,
        path: PathBuf,
        from: Option<ObjectId>,
    ) -> anyhow::Result<FileLineage> {
        let head_id = match from {
            Some(id) => id,
            None => self.repo.head_commit_id()?,
        };

        // The path the file had at the most recent commit we have seen. Walking
        // backward, a rename `from → current` makes this `from`.
        let mut current = path;
        let mut history: Vec<FileLineageNode> = Vec::new();

        for commit_id_res in self.repo.rev_walk(head_id)? {
            let commit_id = commit_id_res?;
            let commit = self.repo.find_commit(commit_id)?;

            // Root commit: the file exists in this tree → Added (its birth),
            // possibly copied from an existing file with identical content.
            if commit.parents.is_empty() {
                if self.repo.blob_at_path(commit.tree_id, &current)?.is_some() {
                    history.push(FileLineageNode {
                        commit_id,
                        path: current.clone(),
                        action: FileAction::Added {
                            copy_of: copy_source(self.repo, commit.tree_id, &current),
                        },
                    });
                }
                break;
            }

            let parent = self.repo.find_commit(commit.parents[0])?;
            let changes = self
                .repo
                .diff_tree_to_tree(Some(parent.tree_id), commit.tree_id)?;

            // Changes where `current` is the destination or the source.
            let mut touched: Vec<&gitx_git::models::FileChange> = changes
                .iter()
                .filter(|c| c.path == current || c.old_path.as_ref() == Some(&current))
                .collect();
            touched.sort_by_key(|c| c.old_path.is_some() == (c.path == current));

            let mut renamed = false;
            for change in touched {
                match change.change_type {
                    gitx_git::models::ChangeType::Renamed
                    | gitx_git::models::ChangeType::Copied
                        if change.path == current =>
                    {
                        if let Some(from) = &change.old_path {
                            history.push(FileLineageNode {
                                commit_id,
                                path: current.clone(),
                                action: FileAction::Renamed { from: from.clone() },
                            });
                            current = from.clone();
                            renamed = true;
                        }
                    }
                    gitx_git::models::ChangeType::Added if change.path == current => {
                        history.push(FileLineageNode {
                            commit_id,
                            path: current.clone(),
                            action: FileAction::Added {
                                copy_of: copy_source(self.repo, commit.tree_id, &current),
                            },
                        });
                    }
                    gitx_git::models::ChangeType::Deleted if change.path == current => {
                        history.push(FileLineageNode {
                            commit_id,
                            path: current.clone(),
                            action: FileAction::Deleted,
                        });
                    }
                    gitx_git::models::ChangeType::Modified if change.path == current => {
                        history.push(FileLineageNode {
                            commit_id,
                            path: current.clone(),
                            action: FileAction::Modified,
                        });
                    }
                    _ => {}
                }
            }
            // A rename already rewound `current`; do not also record the
            // destination's modification in the same commit.
            if renamed {
                continue;
            }
        }

        Ok(FileLineage { history })
    }
}

/// When `path`'s blob in `tree_id` is byte-identical (same oid) to another
/// file in the same tree, that other path is a reliable copy source
/// (docs/02 §2 “copies where reliably detectable”; deterministic, read-only).
fn copy_source(repo: &Repository, tree_id: ObjectId, path: &Path) -> Option<PathBuf> {
    let entries = repo.tree_entries(tree_id).ok()?;
    let own = entries.iter().find(|(p, _)| p == path).map(|(_, o)| o)?;
    entries
        .iter()
        .filter(|(p, _)| p != path)
        .find(|(_, o)| o == own)
        .map(|(p, _)| p.clone())
}
