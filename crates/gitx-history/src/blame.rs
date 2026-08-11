use gitx_git::models::{Commit, ObjectId};
use gitx_git::reflog::split_lines;
use std::ops::Range;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct BlameLine {
    pub line_no: usize,
    pub commit_id: ObjectId,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct BlameResult {
    pub path: PathBuf,
    pub lines: Vec<BlameLine>,
}

impl<'a> super::timeline::HistoryService<'a> {
    /// Compute line-level attribution for `path`, i.e. which commit last
    /// introduced each line of the file's current version.
    ///
    /// This is a pure gix implementation (no shelling out to `git blame`):
    /// the file's history is walked oldest → newest and each commit's version
    /// is line-diffed against its predecessor, attributing added/replaced
    /// lines to that commit.
    ///
    /// Merge commits are attributed against their first parent (mainline).
    pub fn blame(&self, path: PathBuf, from: Option<ObjectId>) -> anyhow::Result<BlameResult> {
        let from_id = match from {
            Some(id) => id,
            None => self.repo.head_commit_id()?,
        };
        let from_commit = self.repo.find_commit(from_id)?;

        // 1. Collect the file's history, oldest → newest.
        let mut history: Vec<Commit> = Vec::new();
        for commit_id_res in self.repo.rev_walk(from_id)? {
            let id = commit_id_res?;
            let commit = self.repo.find_commit(id)?;
            if self.commit_touches_path(&commit, &path)? {
                history.push(commit);
            }
        }
        history.reverse();

        if history.is_empty() {
            anyhow::bail!("no history found for {}", path.display());
        }

        // 2. Forward attribution: `origins` is aligned with the file's version
        //    at the most recently processed commit.
        let mut origins: Vec<Option<ObjectId>> = Vec::new();
        let mut previous_bytes: Vec<u8> = Vec::new();
        let mut file_exists = false;

        for commit in &history {
            let new_bytes = self.repo.blob_at_path(commit.tree_id, &path)?;

            let Some(new_bytes) = new_bytes else {
                // File deleted at this commit — attribution restarts.
                origins.clear();
                file_exists = false;
                continue;
            };

            if !file_exists {
                // File created (or re-created) here: every line belongs to this commit.
                let line_count = split_lines(&new_bytes).len();
                origins = vec![Some(commit.id); line_count];
            } else {
                origins = attribute_lines(&previous_bytes, &new_bytes, origins, commit.id);
            }

            previous_bytes = new_bytes;
            file_exists = true;
        }

        // 3. Materialize the current version's lines with their origins.
        let current_bytes = self
            .repo
            .blob_at_path(from_commit.tree_id, &path)?
            .ok_or_else(|| {
                anyhow::anyhow!("{} does not exist at the given commit", path.display())
            })?;
        let current_lines = split_lines(&current_bytes);
        anyhow::ensure!(
            origins.len() == current_lines.len(),
            "internal blame inconsistency: {} origins for {} lines",
            origins.len(),
            current_lines.len()
        );

        let lines = current_lines
            .into_iter()
            .enumerate()
            .map(|(idx, content)| BlameLine {
                line_no: idx + 1,
                commit_id: origins[idx].unwrap_or(from_id),
                content,
            })
            .collect();

        Ok(BlameResult { path, lines })
    }
}

/// Diff `old_bytes` → `new_bytes` and build the new attribution vector.
/// Lines unchanged between hunks carry their old origin forward; lines added
/// or replaced by the diff are attributed to `commit_id`.
fn attribute_lines(
    old_bytes: &[u8],
    new_bytes: &[u8],
    old_origins: Vec<Option<ObjectId>>,
    commit_id: ObjectId,
) -> Vec<Option<ObjectId>> {
    let new_line_count = split_lines(new_bytes).len();
    let mut new_origins = vec![None; new_line_count];

    let input = gix::diff::blob::intern::InternedInput::new(old_bytes, new_bytes);

    let mut old_cursor = 0usize;
    let mut new_cursor = 0usize;

    gix::diff::blob::diff(
        gix::diff::blob::Algorithm::Myers,
        &input,
        |before: Range<u32>, after: Range<u32>| {
            let unchanged = (before.start as usize).saturating_sub(old_cursor);
            for k in 0..unchanged {
                if let Some(origin) = old_origins.get(old_cursor + k).copied().flatten() {
                    new_origins[new_cursor + k] = Some(origin);
                }
            }
            let added = (after.end - after.start) as usize;
            for slot in new_origins
                .iter_mut()
                .skip(new_cursor + unchanged)
                .take(added)
            {
                *slot = Some(commit_id);
            }
            old_cursor = before.end as usize;
            new_cursor += unchanged + added;
        },
    );

    // Tail: unchanged lines after the last hunk.
    while new_cursor < new_origins.len() {
        if let Some(origin) = old_origins.get(old_cursor).copied().flatten() {
            new_origins[new_cursor] = Some(origin);
        }
        new_cursor += 1;
        old_cursor += 1;
    }

    debug_assert_eq!(new_origins.len(), new_line_count);
    new_origins
}

#[cfg(test)]
mod tests {
    use super::attribute_lines;
    use gitx_git::models::ObjectId;

    fn oid(seed: &str) -> ObjectId {
        ObjectId::from_hex(&seed.repeat(40)).expect("valid hex")
    }

    #[test]
    fn unchanged_lines_keep_origin() {
        let old = b"alpha\nbeta\ngamma\n";
        let new = b"alpha\nbeta\ngamma\n";
        let origins = vec![Some(oid("1")), Some(oid("2")), Some(oid("3"))];
        let out = attribute_lines(old, new, origins.clone(), oid("9"));
        assert_eq!(out, origins);
    }

    #[test]
    fn replaced_lines_are_attributed_to_commit() {
        let old = b"alpha\nbeta\ngamma\n";
        let new = b"alpha\nBETA\ngamma\ndelta\n";
        let origins = vec![Some(oid("1")), Some(oid("2")), Some(oid("3"))];
        let out = attribute_lines(old, new, origins, oid("9"));
        // alpha keeps 1, BETA is new (9), gamma keeps 3, delta is new (9)
        assert_eq!(
            out,
            vec![
                Some(oid("1")),
                Some(oid("9")),
                Some(oid("3")),
                Some(oid("9"))
            ]
        );
    }

    #[test]
    fn deletion_removes_lines() {
        let old = b"alpha\nbeta\ngamma\n";
        let new = b"alpha\ngamma\n";
        let origins = vec![Some(oid("1")), Some(oid("2")), Some(oid("3"))];
        let out = attribute_lines(old, new, origins, oid("9"));
        assert_eq!(out, vec![Some(oid("1")), Some(oid("3"))]);
    }

    #[test]
    fn added_file_attributes_everything() {
        let old = b"";
        let new = b"one\ntwo\n";
        let out = attribute_lines(old, new, Vec::new(), oid("7"));
        assert_eq!(out, vec![Some(oid("7")), Some(oid("7"))]);
    }
}
