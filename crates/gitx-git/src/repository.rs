use crate::error::{GitError, Result};
use crate::models::{Branch, Commit, ObjectId, Signature, Tag};
use std::path::Path;

pub struct Repository {
    pub(crate) repo: gix::Repository,
}

impl Repository {
    pub fn discover(path: impl AsRef<Path>) -> Result<Self> {
        let repo = gix::discover(path.as_ref()).map_err(|e| GitError::OpenFailed(e.to_string()))?;
        Ok(Self { repo })
    }

    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let repo = gix::open(path.as_ref()).map_err(|e| GitError::OpenFailed(e.to_string()))?;
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
}


impl Repository {
    pub fn state(&self) -> Option<String> {
        self.repo.state().map(|s| format!("{:?}", s))
    }
}
