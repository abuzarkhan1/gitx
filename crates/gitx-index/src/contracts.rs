use crate::error::IndexerError;
use crate::models::{Commit, Oid, RefInfo};

pub trait GitProvider {
    fn read_refs(&self) -> Result<Vec<RefInfo>, IndexerError>;

    /// The fully-qualified name of the branch HEAD points at (e.g.
    /// `refs/heads/main`), when HEAD is symbolic — used to track the true
    /// HEAD lineage for rewritten-history detection (docs/09 §5).
    fn head_ref_name(&self) -> Result<Option<String>, IndexerError>;

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

    /// Read a metadata key written by a previous scan/refresh (docs/09:
    /// rewritten-history detection and index freshness rely on it).
    fn get_meta(&self, key: &str) -> Result<Option<String>, IndexerError>;
}

pub trait Transaction {
    fn write_commit(&mut self, commit: &Commit) -> Result<(), IndexerError>;
    fn write_ref(&mut self, ref_info: &RefInfo) -> Result<(), IndexerError>;
    fn remove_ref(&mut self, ref_name: &str) -> Result<(), IndexerError>;
    /// Whether a commit is already present in the index. Queried through the
    /// transaction so the implementation may hold exclusive access (e.g. an
    /// open SQLite transaction) without a separate read path.
    fn is_commit_indexed(&self, id: &Oid) -> Result<bool, IndexerError>;
    /// Write a metadata key/value pair (e.g. the last-seen HEAD oid) so the
    /// next refresh can detect rewritten history (docs/09 §5).
    fn write_meta(&mut self, key: &str, value: &str) -> Result<(), IndexerError>;
    fn commit(self: Box<Self>) -> Result<(), IndexerError>;
    fn rollback(self: Box<Self>) -> Result<(), IndexerError>;
}
