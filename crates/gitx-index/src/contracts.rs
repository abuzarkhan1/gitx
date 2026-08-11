use crate::error::IndexerError;
use crate::models::{Commit, Oid, RefInfo};

pub trait GitProvider {
    fn read_refs(&self) -> Result<Vec<RefInfo>, IndexerError>;

    // Returns an iterator of commits in topological order (newest to oldest, or parent-first depending on traversal)
    // To handle large history, it returns an iterator.
    fn walk_commits(
        &self,
        starting_from: &[Oid],
    ) -> Result<Box<dyn Iterator<Item = Result<Commit, IndexerError>> + '_>, IndexerError>;
}

pub trait StorageProvider {
    fn begin_transaction(&self) -> Result<Box<dyn Transaction + '_>, IndexerError>;

    fn get_indexed_refs(&self) -> Result<Vec<RefInfo>, IndexerError>;
}

pub trait Transaction {
    fn write_commit(&mut self, commit: &Commit) -> Result<(), IndexerError>;
    fn write_ref(&mut self, ref_info: &RefInfo) -> Result<(), IndexerError>;
    fn remove_ref(&mut self, ref_name: &str) -> Result<(), IndexerError>;
    /// Whether a commit is already present in the index. Queried through the
    /// transaction so the implementation may hold exclusive access (e.g. an
    /// open SQLite transaction) without a separate read path.
    fn is_commit_indexed(&self, id: &Oid) -> Result<bool, IndexerError>;
    fn commit(self: Box<Self>) -> Result<(), IndexerError>;
    fn rollback(self: Box<Self>) -> Result<(), IndexerError>;
}
