use std::fmt;

#[derive(Debug)]
pub enum IndexerError {
    GitError(String),
    StorageError(String),
    Cancelled,
    Other(String),
}

impl fmt::Display for IndexerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IndexerError::GitError(s) => write!(f, "Git error: {}", s),
            IndexerError::StorageError(s) => write!(f, "Storage error: {}", s),
            IndexerError::Cancelled => write!(f, "Indexing cancelled"),
            IndexerError::Other(s) => write!(f, "Error: {}", s),
        }
    }
}

impl std::error::Error for IndexerError {}
