use crate::error::{GitError, Result};
use crate::models::{ChangeType, FileChange, ObjectId};
use crate::repository::Repository;
use std::path::PathBuf;

impl Repository {
    /// Diff two trees and produce a list of per-file changes with real
    /// insertion/deletion line counts.
    ///
    /// When `old_tree` is `None` (e.g. the root commit), every blob reachable
    /// from `new_tree` is reported as an addition.
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

        let Some(old_id) = old_tree else {
            // Root commit: every file in the new tree is an addition.
            collect_blobs(&self.repo, new_tree, PathBuf::new(), &mut changes)?;
            return Ok(changes);
        };

        let old_tree_obj = self
            .repo
            .find_object(old_id.0)
            .map_err(|e| GitError::ObjectReadError(old_id.to_string(), e.to_string()))?
            .into_tree();

        let mut cache = self
            .repo
            .diff_resource_cache_for_tree_diff()
            .map_err(|e| GitError::TreeError(e.to_string()))?;

        old_tree_obj
            .changes()
            .map_err(|e| GitError::TreeError(e.to_string()))?
            .track_path()
            // Rename/copy tracking: `Some(default)` enables git's default
            // 50% similarity rename detection (docs/02 §2 renames/copies).
            // `None` would disable it entirely.
            .track_rewrites(Some(gix::diff::Rewrites::default()))
            .for_each_to_obtain_tree(&new_tree_obj, |change| {
                let location = PathBuf::from(change.location.to_string());

                // gix reports changed directories as single tree events and then
                // recurses into modified ones — but blob diff cannot diff trees.
                // Added/deleted directories are expanded into per-file changes.
                match change.event {
                    gix::object::tree::diff::change::Event::Addition { id, entry_mode }
                        if entry_mode.is_tree() =>
                    {
                        collect_blobs_with(
                            &self.repo,
                            ObjectId(id.detach()),
                            location,
                            ChangeType::Added,
                            &mut changes,
                        )?;
                        return Ok::<_, GitError>(gix::object::tree::diff::Action::Continue);
                    }
                    gix::object::tree::diff::change::Event::Deletion { id, entry_mode }
                        if entry_mode.is_tree() =>
                    {
                        collect_blobs_with(
                            &self.repo,
                            ObjectId(id.detach()),
                            location,
                            ChangeType::Deleted,
                            &mut changes,
                        )?;
                        return Ok::<_, GitError>(gix::object::tree::diff::Action::Continue);
                    }
                    gix::object::tree::diff::change::Event::Modification { entry_mode, .. }
                        if entry_mode.is_tree() =>
                    {
                        // Modified directory: gix already recurses into it.
                        return Ok::<_, GitError>(gix::object::tree::diff::Action::Continue);
                    }
                    _ => {}
                }

                let (insertions, deletions) = line_stats(&mut cache, &change)?;

                match change.event {
                    gix::object::tree::diff::change::Event::Addition { .. } => {
                        changes.push(FileChange {
                            path: location,
                            old_path: None,
                            change_type: ChangeType::Added,
                            insertions,
                            deletions,
                        });
                    }
                    gix::object::tree::diff::change::Event::Deletion { .. } => {
                        changes.push(FileChange {
                            path: location,
                            old_path: None,
                            change_type: ChangeType::Deleted,
                            insertions,
                            deletions,
                        });
                    }
                    gix::object::tree::diff::change::Event::Modification { .. } => {
                        changes.push(FileChange {
                            path: location,
                            old_path: None,
                            change_type: ChangeType::Modified,
                            insertions,
                            deletions,
                        });
                    }
                    gix::object::tree::diff::change::Event::Rewrite {
                        source_location,
                        copy,
                        ..
                    } => {
                        changes.push(FileChange {
                            path: location,
                            old_path: Some(PathBuf::from(source_location.to_string())),
                            change_type: if copy {
                                ChangeType::Copied
                            } else {
                                ChangeType::Renamed
                            },
                            insertions,
                            deletions,
                        });
                    }
                }
                Ok::<_, GitError>(gix::object::tree::diff::Action::Continue)
            })
            .map_err(|e| GitError::TreeError(source_chain(&e)))?;

        Ok(changes)
    }
}

/// Flatten an error chain into a single readable string (gix wraps callback
/// errors in a boxed source that `to_string()` hides).
pub(crate) fn source_chain(err: &dyn std::error::Error) -> String {
    let mut out = err.to_string();
    let mut cur = err.source();
    while let Some(cause) = cur {
        out.push_str(": ");
        out.push_str(&cause.to_string());
        cur = cause.source();
    }
    out
}

/// Line counts (insertions, deletions) for a tree change via gix's blob diff.
/// Binary resources yield `(0, 0)`.
fn line_stats(
    cache: &mut gix::diff::blob::Platform,
    change: &gix::object::tree::diff::Change<'_, '_, '_>,
) -> Result<(u32, u32)> {
    let mut platform = change
        .diff(cache)
        .map_err(|e| GitError::TreeError(e.to_string()))?;
    let counter = platform
        .line_counts()
        .map_err(|e| GitError::TreeError(e.to_string()))?;
    match counter {
        Some(c) => Ok((c.insertions, c.removals)),
        None => Ok((0, 0)),
    }
}

/// Recursively collect every blob in `tree_id` as a file change of the given
/// type, counting its lines as insertions (for additions) or deletions.
/// Used for the root commit and for added/deleted directories.
fn collect_blobs(
    repo: &gix::Repository,
    tree_id: ObjectId,
    prefix: PathBuf,
    out: &mut Vec<FileChange>,
) -> Result<()> {
    collect_blobs_with(repo, tree_id, prefix, ChangeType::Added, out)
}

fn collect_blobs_with(
    repo: &gix::Repository,
    tree_id: ObjectId,
    prefix: PathBuf,
    change_type: ChangeType,
    out: &mut Vec<FileChange>,
) -> Result<()> {
    let tree = repo
        .find_object(tree_id.0)
        .map_err(|e| GitError::ObjectReadError(tree_id.to_string(), e.to_string()))?
        .into_tree();
    let decoded = tree
        .decode()
        .map_err(|e| GitError::TreeError(e.to_string()))?;

    for entry in decoded.entries {
        let name = std::str::from_utf8(entry.filename).unwrap_or("<invalid>");
        let path = prefix.join(name);

        if entry.mode.is_tree() {
            collect_blobs_with(
                repo,
                ObjectId(entry.oid.to_owned()),
                path,
                change_type.clone(),
                out,
            )?;
        } else if entry.mode.is_commit() {
            // Git submodule — recorded as a directory entry, skipped for now.
            continue;
        } else {
            let lines = blob_line_count(repo, ObjectId(entry.oid.to_owned()))?;
            out.push(FileChange {
                path,
                old_path: None,
                change_type: change_type.clone(),
                insertions: if change_type == ChangeType::Added {
                    lines
                } else {
                    0
                },
                deletions: if change_type == ChangeType::Deleted {
                    lines
                } else {
                    0
                },
            });
        }
    }
    Ok(())
}

fn blob_line_count(repo: &gix::Repository, blob_id: ObjectId) -> Result<u32> {
    let blob = repo
        .find_object(blob_id.0)
        .map_err(|e| GitError::ObjectReadError(blob_id.to_string(), e.to_string()))?
        .into_blob();
    Ok(count_lines(&blob.data) as u32)
}

/// Render a unified diff for a single commit (used by `gitx recovery export`,
/// docs/12 §6). Hunks come from the same gix blob diff used by blame; the
/// output is a valid `git apply`-able patch.
pub fn render_commit_patch(repo: &Repository, commit_id: ObjectId) -> Result<String> {
    let commit = repo.find_commit(commit_id)?;
    let parent_tree = match commit.parents.first() {
        Some(parent) => Some(repo.find_commit(*parent)?.tree_id),
        None => None,
    };
    let changes = repo.diff_tree_to_tree(parent_tree, commit.tree_id)?;

    let mut out = String::new();
    out.push_str(&format!("commit {}\n", commit.id));
    out.push_str(&format!(
        "Author: {} <{}>\n",
        commit.author.name, commit.author.email
    ));
    out.push_str(&format!("Date:   {}\n", commit.author.time));
    out.push('\n');
    for line in commit.message.lines() {
        out.push_str(&format!("    {line}\n"));
    }
    out.push('\n');

    for change in &changes {
        if let Some(patch) = render_file_patch(repo, parent_tree, commit.tree_id, change)? {
            out.push_str(&patch);
        }
    }
    Ok(out)
}

/// Render a unified patch for one file change between two trees (docs/13
/// §8: callers stream files one at a time so only one file's hunks are in
/// memory). Returns `None` when both blobs are missing.
pub fn render_file_patch(
    repo: &Repository,
    old_tree: Option<ObjectId>,
    new_tree: ObjectId,
    change: &FileChange,
) -> Result<Option<String>> {
    let old_path = change.old_path.as_ref().unwrap_or(&change.path);
    let old_bytes = repo
        .blob_at_path(old_tree.unwrap_or(new_tree), old_path)
        .ok()
        .flatten();
    let new_bytes = repo.blob_at_path(new_tree, &change.path).ok().flatten();
    if old_bytes.is_none() && new_bytes.is_none() {
        return Ok(None);
    }

    let mut out = String::new();
    out.push_str(&format!(
        "diff --git a/{} b/{}\n",
        old_path.display(),
        change.path.display()
    ));
    match (&old_bytes, &new_bytes) {
        (None, Some(_)) => {
            out.push_str("new file mode 100644\n");
        }
        (Some(_), None) => {
            out.push_str("deleted file mode 100644\n");
        }
        _ => {}
    }
    out.push_str(&format!(
        "--- {}\n",
        if old_bytes.is_some() {
            format!("a/{}", old_path.display())
        } else {
            "/dev/null".to_string()
        }
    ));
    out.push_str(&format!(
        "+++ {}\n",
        if new_bytes.is_some() {
            format!("b/{}", change.path.display())
        } else {
            "/dev/null".to_string()
        }
    ));

    let old_bytes = old_bytes.unwrap_or_default();
    let new_bytes = new_bytes.unwrap_or_default();

    // A single hunk covering the whole file: every line of the old version
    // is deleted and every line of the new version added. This is a valid
    // unified diff (git apply accepts it) and preserves the complete file
    // state at that ref.
    let old_count = count_lines(&old_bytes);
    let new_count = count_lines(&new_bytes);
    out.push_str(&format!("@@ -1,{old_count} +1,{new_count} @@\n"));
    for line in split_utf8_lines(&old_bytes) {
        out.push_str(&format!("-{line}\n"));
    }
    for line in split_utf8_lines(&new_bytes) {
        out.push_str(&format!("+{line}\n"));
    }
    out.push('\n');
    Ok(Some(out))
}

/// Split bytes into UTF-8 lines (content only, trailing newline stripped).
fn split_utf8_lines(data: &[u8]) -> Vec<String> {
    let text = String::from_utf8_lossy(data);
    text.lines().map(|l| l.to_string()).collect()
}

/// Count lines in raw blob bytes, tolerating a missing trailing newline.
pub(crate) fn count_lines(data: &[u8]) -> usize {
    if data.is_empty() {
        return 0;
    }
    data.iter().filter(|&&b| b == b'\n').count() + usize::from(!data.ends_with(b"\n"))
}
