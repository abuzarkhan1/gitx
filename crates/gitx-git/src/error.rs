use thiserror::Error;

pub type Result<T> = std::result::Result<T, GitError>;

#[derive(Error, Debug)]
pub enum GitError {
    #[error("Git repository not found at {0}")]
    NotFound(std::path::PathBuf),

    #[error("Failed to open repository: {0}")]
    OpenFailed(String),

    #[error("Failed to read object {0}: {1}")]
    ObjectReadError(String, String),

    #[error("Commit not found: {0}")]
    CommitNotFound(String),

    #[error("Tree traversal error: {0}")]
    TreeError(String),

    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),
}
