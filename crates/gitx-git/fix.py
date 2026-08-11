with open("src/repository.rs", "r") as f:
    content = f.read()

prefix = """use crate::error::{GitError, Result};
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
"""

# find the part after the missing text
idx = content.find("        let repo = gix::open(path.as_ref())")
if idx != -1:
    new_content = prefix + content[idx:]
    with open("src/repository.rs", "w") as f:
        f.write(new_content)
