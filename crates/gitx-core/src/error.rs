use thiserror::Error;

/// The central Result type for GitX.
pub type Result<T, E = GitxError> = std::result::Result<T, E>;

/// The central error type for the GitX application.
#[derive(Debug, Error)]
pub enum GitxError {
    #[error("Git error: {0}")]
    Git(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid configuration: {0}")]
    Config(String),

    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}
