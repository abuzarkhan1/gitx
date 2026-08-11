use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub path: PathBuf,
    pub size: u64,
    pub modified: SystemTime,
    pub hash: String, // e.g. SHA-256 or Git blob hash
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectorySnapshot {
    pub root: PathBuf,
    pub files: Vec<FileMetadata>,
    pub timestamp: SystemTime,
    pub commit_id: Option<String>,
}

impl DirectorySnapshot {
    pub fn new(root: impl AsRef<Path>, commit_id: Option<String>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
            files: Vec::new(),
            timestamp: SystemTime::now(),
            commit_id,
        }
    }

    pub fn add_file(&mut self, metadata: FileMetadata) {
        self.files.push(metadata);
    }
}
