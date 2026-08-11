use crate::error::{GitError, Result};
use crate::models::ObjectId;
use crate::repository::Repository;
use gix::bstr::ByteSlice;
use std::path::Path;

/// One entry of a reference log (reflog).
#[derive(Debug, Clone)]
pub struct ReflogEntry {
    /// The fully-qualified reference name, e.g. `refs/heads/main` or `HEAD`.
    pub reference: String,
    /// The object id before the update (null hash for the first entry).
    pub previous_oid: ObjectId,
    /// The object id after the update (null hash if the ref was deleted).
    pub new_oid: ObjectId,
    /// The actor who performed the update, if parseable.
    pub actor_name: Option<String>,
    pub actor_email: Option<String>,
    /// Unix timestamp of the update.
    pub timestamp: Option<i64>,
    /// The reflog message (e.g. `commit: fix workspace bug`).
    pub message: String,
}

impl Repository {
    /// Read the reflog for `reference` (e.g. `"HEAD"` or `"refs/heads/main"`),
    /// newest entry first. Returns an empty list when no reflog exists.
    pub fn reflog(&self, reference: &str) -> Result<Vec<ReflogEntry>> {
        let reference = self
            .repo
            .find_reference(reference)
            .map_err(|e| GitError::Other(anyhow::anyhow!(e)))?;

        if !reference.log_exists() {
            return Ok(Vec::new());
        }

        let mut platform = reference.log_iter();
        let Some(mut iter) = platform
            .rev()
            .map_err(|e| GitError::Other(anyhow::anyhow!(e)))?
        else {
            return Ok(Vec::new());
        };

        let mut entries = Vec::new();
        for line in iter.by_ref() {
            let line = line.map_err(|e| GitError::Other(anyhow::anyhow!(e)))?;
            entries.push(ReflogEntry {
                reference: reference.name().as_bstr().to_string(),
                previous_oid: ObjectId(line.previous_oid),
                new_oid: ObjectId(line.new_oid),
                actor_name: line.signature.name.to_str().ok().map(str::to_owned),
                actor_email: line.signature.email.to_str().ok().map(str::to_owned),
                timestamp: Some(line.signature.time.seconds),
                message: line.message.to_string(),
            });
        }
        Ok(entries)
    }

    /// Read the reflog of `HEAD`, newest first.
    pub fn head_reflog(&self) -> Result<Vec<ReflogEntry>> {
        self.reflog("HEAD")
    }

    /// Return the raw bytes of the blob at `path` in the tree identified by
    /// `tree_id`, or `None` if the path does not exist in that tree.
    pub fn blob_at_path(&self, tree_id: ObjectId, path: &Path) -> Result<Option<Vec<u8>>> {
        let tree = self
            .repo
            .find_object(tree_id.0)
            .map_err(|e| GitError::ObjectReadError(tree_id.to_string(), e.to_string()))?
            .into_tree();

        let mut buf = Vec::new();
        let Some(entry) = tree
            .lookup_entry_by_path(path, &mut buf)
            .map_err(|e| GitError::TreeError(e.to_string()))?
        else {
            return Ok(None);
        };

        let object = entry
            .object()
            .map_err(|e| GitError::ObjectReadError(path.display().to_string(), e.to_string()))?;
        if object.kind != gix::objs::Kind::Blob {
            return Ok(None);
        }
        Ok(Some(object.data.clone()))
    }
}

/// Split raw blob bytes into lines (terminators stripped). The final line is
/// included even without a trailing newline.
pub fn split_lines(data: &[u8]) -> Vec<String> {
    if data.is_empty() {
        return Vec::new();
    }
    let mut lines: Vec<String> = data
        .split(|&b| b == b'\n')
        .map(|line| String::from_utf8_lossy(line).into_owned())
        .collect();
    // `split` yields a trailing empty element when data ends with '\n'.
    if data.ends_with(b"\n") {
        lines.pop();
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::split_lines;

    #[test]
    fn split_lines_handles_trailing_newline() {
        assert_eq!(split_lines(b"a\nb\n"), vec!["a", "b"]);
    }

    #[test]
    fn split_lines_handles_missing_trailing_newline() {
        assert_eq!(split_lines(b"a\nb"), vec!["a", "b"]);
    }

    #[test]
    fn split_lines_empty() {
        assert!(split_lines(b"").is_empty());
    }
}
