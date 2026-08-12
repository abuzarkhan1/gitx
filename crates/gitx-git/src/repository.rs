use crate::error::{GitError, Result};
use crate::models::{Branch, Commit, ObjectId, Signature, Tag};
use std::path::Path;

pub struct Repository {
    pub(crate) repo: gix::Repository,
}

impl Clone for Repository {
    /// Cheap handle clone sharing the underlying object database. Each clone
    /// gets its own cache slots, so cloning per worker thread is the
    /// recommended gix pattern for parallel reads (docs/13 §6).
    fn clone(&self) -> Self {
        Self {
            repo: self.repo.clone(),
        }
    }
}

/// Decoded-object cache budget (docs/13 §4 bounded caches). gix leaves its
/// cache unset by default; history walks and per-commit tree diffs then
/// re-decompress the same pack objects on every access (zlib-dominated). A
/// memory-capped cache that grows gradually turns repeated reads into hash
/// lookups; it is shared across Repository clones (each clone gets cache
/// slots over the same object store).
const OBJECT_CACHE_BYTES: usize = 128 * 1024 * 1024;

impl Repository {
    pub fn discover(path: impl AsRef<Path>) -> Result<Self> {
        let started = std::time::Instant::now();
        let mut repo =
            gix::discover(path.as_ref()).map_err(|e| GitError::OpenFailed(e.to_string()))?;
        repo.object_cache_size_if_unset(OBJECT_CACHE_BYTES);
        tracing::debug!(
            path = %path.as_ref().display(),
            git_dir = %repo.git_dir().display(),
            elapsed_ms = started.elapsed().as_millis() as u64,
            "repository discovered"
        );
        Ok(Self { repo })
    }

    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let mut repo = gix::open(path.as_ref()).map_err(|e| GitError::OpenFailed(e.to_string()))?;
        repo.object_cache_size_if_unset(OBJECT_CACHE_BYTES);
        Ok(Self { repo })
    }

    pub fn work_dir(&self) -> Option<&Path> {
        self.repo.work_dir()
    }

    pub fn git_dir(&self) -> &Path {
        self.repo.git_dir()
    }

    pub fn head_commit_id(&self) -> Result<ObjectId> {
        let head = self
            .repo
            .head()
            .map_err(|e| GitError::Other(anyhow::anyhow!(e)))?;
        let commit = head
            .into_peeled_id()
            .map_err(|e| GitError::Other(anyhow::anyhow!(e)))?;
        Ok(ObjectId(commit.detach()))
    }

    pub fn find_commit(&self, id: ObjectId) -> Result<Commit> {
        let commit = self
            .repo
            .find_object(id.0)
            .map_err(|e| GitError::ObjectReadError(id.to_string(), e.to_string()))?
            .into_commit();

        let author = commit
            .author()
            .map_err(|e| GitError::Other(anyhow::anyhow!(e)))?;
        let committer = commit
            .committer()
            .map_err(|e| GitError::Other(anyhow::anyhow!(e)))?;
        let message = commit
            .message()
            .map_err(|e| GitError::Other(anyhow::anyhow!(e)))?
            .title
            .to_string();

        Ok(Commit {
            id,
            tree_id: ObjectId(
                commit
                    .tree_id()
                    .map_err(|e| GitError::Other(anyhow::anyhow!(e)))?
                    .into(),
            ),
            parents: commit.parent_ids().map(|id| ObjectId(id.into())).collect(),
            author: Signature {
                name: author.name.to_string(),
                email: author.email.to_string(),
                time: author.time.seconds,
                offset: author.time.offset,
            },
            committer: Signature {
                name: committer.name.to_string(),
                email: committer.email.to_string(),
                time: committer.time.seconds,
                offset: committer.time.offset,
            },
            message,
        })
    }

    pub fn branches(&self) -> Result<Vec<Branch>> {
        let mut branches = Vec::new();
        let refs = self
            .repo
            .references()
            .map_err(|e| GitError::Other(anyhow::anyhow!(e)))?;
        for r in refs
            .local_branches()
            .map_err(|e| GitError::Other(anyhow::anyhow!(e)))?
        {
            let r = r.map_err(|e| GitError::Other(anyhow::anyhow!(e)))?;
            let name_tmp = r.name().shorten().to_string();
            let target = r
                .into_fully_peeled_id()
                .map_err(|e| GitError::Other(anyhow::anyhow!(e)))?;
            let name = name_tmp;
            branches.push(Branch {
                name,
                target: ObjectId(target.detach()),
                is_remote: false,
            });
        }
        for r in refs
            .remote_branches()
            .map_err(|e| GitError::Other(anyhow::anyhow!(e)))?
        {
            let r = r.map_err(|e| GitError::Other(anyhow::anyhow!(e)))?;
            let name_tmp = r.name().shorten().to_string();
            let target = r
                .into_fully_peeled_id()
                .map_err(|e| GitError::Other(anyhow::anyhow!(e)))?;
            let name = name_tmp;
            branches.push(Branch {
                name,
                target: ObjectId(target.detach()),
                is_remote: true,
            });
        }
        Ok(branches)
    }

    pub fn tags(&self) -> Result<Vec<Tag>> {
        let mut tags = Vec::new();
        let refs = self
            .repo
            .references()
            .map_err(|e| GitError::Other(anyhow::anyhow!(e)))?;
        for r in refs
            .tags()
            .map_err(|e| GitError::Other(anyhow::anyhow!(e)))?
        {
            let r = r.map_err(|e| GitError::Other(anyhow::anyhow!(e)))?;
            let name_tmp = r.name().shorten().to_string();
            let target = r
                .into_fully_peeled_id()
                .map_err(|e| GitError::Other(anyhow::anyhow!(e)))?;
            let name = name_tmp;
            tags.push(Tag {
                name,
                target: ObjectId(target.detach()),
            });
        }
        Ok(tags)
    }

    pub fn rev_walk<'a>(
        &'a self,
        head: ObjectId,
    ) -> Result<impl Iterator<Item = Result<ObjectId>> + 'a> {
        let rev_walk = self.repo.rev_walk([head.0]);
        let iter = rev_walk
            .all()
            .map_err(|e| GitError::Other(anyhow::anyhow!(e)))?;
        Ok(iter.map(|info_res| {
            info_res
                .map(|info| ObjectId(info.id))
                .map_err(|e| GitError::Other(anyhow::anyhow!(e)))
        }))
    }
}

impl Repository {
    pub fn state(&self) -> Option<String> {
        self.repo.state().map(|s| format!("{:?}", s))
    }

    /// The fully-qualified name of the branch HEAD points at (e.g.
    /// `refs/heads/main`), when HEAD is symbolic. Used by the indexer to
    /// track the true HEAD lineage for rewritten-history detection (docs/09
    /// §5), since branch iteration order is otherwise arbitrary.
    pub fn head_ref_name(&self) -> Result<Option<String>> {
        match self.repo.head_name() {
            Ok(Some(name)) => Ok(Some(name.to_string())),
            Ok(None) => Ok(None),
            Err(_) => Ok(None),
        }
    }

    /// Recursively list every blob path (file) reachable from `tree_id`.
    /// Submodules are skipped.
    pub fn list_blobs(&self, tree_id: ObjectId) -> Result<Vec<std::path::PathBuf>> {
        let mut out = Vec::new();
        collect_paths(&self.repo, tree_id, std::path::PathBuf::new(), &mut out)?;
        Ok(out)
    }

    /// Recursively list every blob path and its object id reachable from
    /// `tree_id` (used by architecture diffing). Submodules are skipped.
    pub fn tree_entries(&self, tree_id: ObjectId) -> Result<Vec<(std::path::PathBuf, ObjectId)>> {
        let mut out = Vec::new();
        collect_entries(&self.repo, tree_id, std::path::PathBuf::new(), &mut out)?;
        Ok(out)
    }

    /// Every tree object id reachable from `tree_id`, including the root
    /// itself (used by recovery to classify dangling trees, docs/12).
    pub fn tree_oids(&self, tree_id: ObjectId) -> Result<Vec<ObjectId>> {
        let mut out = Vec::new();
        collect_trees(&self.repo, tree_id, &mut out)?;
        Ok(out)
    }
}

fn collect_trees(repo: &gix::Repository, tree_id: ObjectId, out: &mut Vec<ObjectId>) -> Result<()> {
    out.push(tree_id);
    let tree = repo
        .find_object(tree_id.0)
        .map_err(|e| GitError::ObjectReadError(tree_id.to_string(), e.to_string()))?
        .into_tree();
    let decoded = tree
        .decode()
        .map_err(|e| GitError::TreeError(e.to_string()))?;
    for entry in decoded.entries {
        if entry.mode.is_tree() {
            collect_trees(repo, ObjectId(entry.oid.to_owned()), out)?;
        }
    }
    Ok(())
}

fn collect_entries(
    repo: &gix::Repository,
    tree_id: ObjectId,
    prefix: std::path::PathBuf,
    out: &mut Vec<(std::path::PathBuf, ObjectId)>,
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
        let oid = ObjectId(entry.oid.to_owned());
        if entry.mode.is_tree() {
            collect_entries(repo, oid, path, out)?;
        } else if entry.mode.is_commit() {
            continue; // submodule
        } else {
            out.push((path, oid));
        }
    }
    Ok(())
}

fn collect_paths(
    repo: &gix::Repository,
    tree_id: ObjectId,
    prefix: std::path::PathBuf,
    out: &mut Vec<std::path::PathBuf>,
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
            collect_paths(repo, ObjectId(entry.oid.to_owned()), path, out)?;
        } else if entry.mode.is_commit() {
            continue; // submodule
        } else {
            out.push(path);
        }
    }
    Ok(())
}

/// The kind of a Git object.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectKind {
    Commit,
    Tree,
    Blob,
    Tag,
}

impl Repository {
    /// The kind of object `id` refers to, or `None` if it does not exist.
    pub fn object_kind(&self, id: ObjectId) -> Result<Option<ObjectKind>> {
        let object = match self.repo.find_object(id.0) {
            Ok(o) => o,
            Err(gix::object::find::existing::Error::NotFound { .. }) => return Ok(None),
            Err(e) => return Err(GitError::ObjectReadError(id.to_string(), e.to_string())),
        };
        Ok(Some(match object.kind {
            gix::objs::Kind::Commit => ObjectKind::Commit,
            gix::objs::Kind::Tree => ObjectKind::Tree,
            gix::objs::Kind::Blob => ObjectKind::Blob,
            gix::objs::Kind::Tag => ObjectKind::Tag,
        }))
    }

    /// Iterate over every object id in the object database (packs and loose).
    /// Used by recovery analysis to find commits not reachable from any ref.
    pub fn all_object_ids(
        &self,
    ) -> crate::Result<Box<dyn Iterator<Item = crate::Result<ObjectId>> + '_>> {
        let iter = self
            .repo
            .objects
            .iter()
            .map_err(|e| GitError::Other(anyhow::anyhow!(e)))?;
        Ok(Box::new(iter.map(|res| {
            res.map(ObjectId)
                .map_err(|e| GitError::Other(anyhow::anyhow!(e)))
        })))
    }
}
