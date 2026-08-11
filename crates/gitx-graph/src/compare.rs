use crate::snapshot::DirectorySnapshot;
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct StructuralDiff {
    pub added: Vec<PathBuf>,
    pub removed: Vec<PathBuf>,
    pub modified: Vec<PathBuf>,
}

pub fn compare_snapshots(old: &DirectorySnapshot, new: &DirectorySnapshot) -> StructuralDiff {
    let mut old_files = HashSet::new();
    let mut new_files = HashSet::new();
    let mut old_hashes = std::collections::HashMap::new();
    let mut new_hashes = std::collections::HashMap::new();

    for file in &old.files {
        old_files.insert(file.path.clone());
        old_hashes.insert(file.path.clone(), file.hash.clone());
    }

    for file in &new.files {
        new_files.insert(file.path.clone());
        new_hashes.insert(file.path.clone(), file.hash.clone());
    }

    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut modified = Vec::new();

    for path in new_files.difference(&old_files) {
        added.push(path.clone());
    }

    for path in old_files.difference(&new_files) {
        removed.push(path.clone());
    }

    for path in old_files.intersection(&new_files) {
        if old_hashes.get(path) != new_hashes.get(path) {
            modified.push(path.clone());
        }
    }

    StructuralDiff {
        added,
        removed,
        modified,
    }
}
