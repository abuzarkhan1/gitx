use crate::error::IndexerError;
use crate::models::{Commit, Oid, RefInfo};

pub trait GitProvider {
    fn read_refs(&self) -> Result<Vec<RefInfo>, IndexerError>;

    /// The fully-qualified name of the branch HEAD points at (e.g.
    /// `refs/heads/main`), when HEAD is symbolic — used to track the true
    /// HEAD lineage for rewritten-history detection (docs/09 §5).
    fn head_ref_name(&self) -> Result<Option<String>, IndexerError>;

    // Returns an iterator of commits the index needs (newest to oldest),
    // stopping at commits already present in `indexed` — their ancestry is by
    // construction indexed too, so the provider must not descend into them
    // (docs/13 §3). This is what keeps incremental refresh O(new commits) on
    // large repositories instead of re-walking the whole history.
    //
    // `indexed` is the set of commit oids already present in the storage,
    // loaded once by the engine. The returned iterator borrows both `self`
    // and `indexed`, so its lifetime `'w` is the shared region of the two.
    fn walk_commits<'s, 'i, 'w>(
        &'s self,
        starting_from: &[Oid],
        indexed: &'i std::collections::HashSet<String>,
    ) -> Result<Box<dyn Iterator<Item = Result<Commit, IndexerError>> + 'w>, IndexerError>
    where
        's: 'w,
        'i: 'w;
}

pub trait StorageProvider {
    fn begin_transaction(&self) -> Result<Box<dyn Transaction + '_>, IndexerError>;

    fn get_indexed_refs(&self) -> Result<Vec<RefInfo>, IndexerError>;

    /// Every commit oid currently in the index, loaded once per scan/refresh
    /// (docs/13 §3): the engine passes it to [`GitProvider::walk_commits`] so
    /// already-indexed commits are never re-read from the object database.
    fn get_indexed_oids(&self) -> Result<std::collections::HashSet<String>, IndexerError>;

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
