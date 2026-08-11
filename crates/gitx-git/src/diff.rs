use crate::error::{GitError, Result};
use crate::models::{ChangeType, FileChange, ObjectId};
use crate::repository::Repository;
use std::path::PathBuf;

impl Repository {
    pub fn diff_tree_to_tree(
        &self,
        old_tree: Option<ObjectId>,
        new_tree: ObjectId,
    ) -> Result<Vec<FileChange>> {
        let new_tree_obj = self
            .repo
            .find_object(new_tree.0)
            .map_err(|e| GitError::ObjectReadError(new_tree.to_string(), e.to_string()))?
            .into_tree();

        let mut changes = Vec::new();

        if let Some(old_id) = old_tree {
            let old_tree_obj = self
                .repo
                .find_object(old_id.0)
                .map_err(|e| GitError::ObjectReadError(old_id.to_string(), e.to_string()))?
                .into_tree();

            let delegate = DiffDelegate {
                changes: &mut changes,
            };

            old_tree_obj
                .changes()
                .map_err(|e| GitError::TreeError(e.to_string()))?
                .track_path()
                .track_rewrites(None)
                .for_each_to_obtain_tree(&new_tree_obj, |change| {
                    match change.event {
                        gix::object::tree::diff::change::Event::Addition { .. } => {
                            delegate.changes.push(FileChange {
                                path: PathBuf::from(change.location.to_string()),
                                old_path: None,
                                change_type: ChangeType::Added,
                                insertions: 0,
                                deletions: 0,
                            });
                        }
                        gix::object::tree::diff::change::Event::Deletion { .. } => {
                            delegate.changes.push(FileChange {
                                path: PathBuf::from(change.location.to_string()),
                                old_path: None,
                                change_type: ChangeType::Deleted,
                                insertions: 0,
                                deletions: 0,
                            });
                        }
                        gix::object::tree::diff::change::Event::Modification { .. } => {
                            delegate.changes.push(FileChange {
                                path: PathBuf::from(change.location.to_string()),
                                old_path: None,
                                change_type: ChangeType::Modified,
                                insertions: 0,
                                deletions: 0,
                            });
                        }
                        _ => {}
                    }
                    Ok::<_, std::convert::Infallible>(gix::object::tree::diff::Action::Continue)
                })
                .map_err(|e| GitError::TreeError(e.to_string()))?;
        }

        Ok(changes)
    }
}

struct DiffDelegate<'a> {
    changes: &'a mut Vec<FileChange>,
}
