use gitx_index::contracts::{StorageProvider, Transaction};
use gitx_index::error::IndexerError;
use gitx_index::models::{Commit, Oid, RefInfo};
use std::cell::{RefCell, RefMut};
use std::result::Result;

/// SQLite-backed [`StorageProvider`] for the incremental indexer (docs/09).
///
/// Wraps a `RefCell<rusqlite::Connection>` so read-only queries and writes can
/// be interleaved through `&self` (the trait contract has no `&mut` access).
///
/// Transactions use explicit `BEGIN`/`COMMIT` (rather than
/// `rusqlite::Transaction`) because a `rusqlite::Transaction` borrows the
/// connection through the `RefMut` guard, which the borrow checker cannot
/// prove outlives the call.
pub struct SqliteStorageProvider<'a> {
    conn: &'a RefCell<rusqlite::Connection>,
}

impl<'a> SqliteStorageProvider<'a> {
    pub fn new(conn: &'a RefCell<rusqlite::Connection>) -> Self {
        Self { conn }
    }
}

impl StorageProvider for SqliteStorageProvider<'_> {
    fn begin_transaction(&self) -> Result<Box<dyn Transaction + '_>, IndexerError> {
        let guard = self.conn.borrow_mut();
        guard
            .execute_batch("BEGIN IMMEDIATE")
            .map_err(|e| IndexerError::StorageError(e.to_string()))?;
        Ok(Box::new(SqliteTransaction {
            conn: guard,
            active: true,
        }))
    }

    fn get_indexed_refs(&self) -> Result<Vec<RefInfo>, IndexerError> {
        let conn = self.conn.borrow();
        let mut refs = Vec::new();

        let mut stmt = conn
            .prepare("SELECT name, tip_oid, is_remote FROM branches")
            .map_err(|e| IndexerError::StorageError(e.to_string()))?;
        let rows = stmt
            .query_map([], |row| {
                let name: String = row.get(0)?;
                let tip: String = row.get(1)?;
                let is_remote: i64 = row.get(2)?;
                Ok((name, tip, is_remote))
            })
            .map_err(|e| IndexerError::StorageError(e.to_string()))?;
        for row in rows {
            let (name, tip, is_remote) =
                row.map_err(|e| IndexerError::StorageError(e.to_string()))?;
            let prefixed = if is_remote != 0 {
                format!("refs/remotes/{name}")
            } else {
                format!("refs/heads/{name}")
            };
            refs.push(RefInfo {
                name: prefixed,
                target: Oid(tip),
            });
        }

        let mut stmt = conn
            .prepare("SELECT name, target_oid FROM tags")
            .map_err(|e| IndexerError::StorageError(e.to_string()))?;
        let rows = stmt
            .query_map([], |row| {
                let name: String = row.get(0)?;
                let target: String = row.get(1)?;
                Ok((name, target))
            })
            .map_err(|e| IndexerError::StorageError(e.to_string()))?;
        for row in rows {
            let (name, target) = row.map_err(|e| IndexerError::StorageError(e.to_string()))?;
            refs.push(RefInfo {
                name: format!("refs/tags/{name}"),
                target: Oid(target),
            });
        }
        Ok(refs)
    }
}

struct SqliteTransaction<'a> {
    /// Guards the connection for the transaction's lifetime; the guard also
    /// provides the `&mut` needed for BEGIN/COMMIT.
    conn: RefMut<'a, rusqlite::Connection>,
    active: bool,
}

impl Drop for SqliteTransaction<'_> {
    fn drop(&mut self) {
        if self.active {
            let _ = self.conn.execute_batch("ROLLBACK");
        }
    }
}

impl Transaction for SqliteTransaction<'_> {
    fn write_commit(&mut self, commit: &Commit) -> Result<(), IndexerError> {
        self.conn
            .execute(
                "INSERT OR IGNORE INTO commits (oid, tree_oid, timestamp, message) VALUES (?1, NULL, ?2, ?3)",
                rusqlite::params![
                    commit.id.0,
                    commit.timestamp.unwrap_or(0),
                    commit.message.as_deref().unwrap_or("")
                ],
            )
            .map_err(|e| IndexerError::StorageError(e.to_string()))?;
        for (idx, parent) in commit.parents.iter().enumerate() {
            self.conn
                .execute(
                    "INSERT OR IGNORE INTO commit_parents (commit_oid, parent_oid, parent_index) VALUES (?1, ?2, ?3)",
                    rusqlite::params![commit.id.0, parent.0, idx as i64],
                )
                .map_err(|e| IndexerError::StorageError(e.to_string()))?;
        }
        Ok(())
    }

    fn write_ref(&mut self, ref_info: &RefInfo) -> Result<(), IndexerError> {
        if let Some(name) = ref_info.name.strip_prefix("refs/tags/") {
            self.conn
                .execute(
                    "INSERT OR IGNORE INTO tags (name, target_oid) VALUES (?1, ?2)",
                    rusqlite::params![name, ref_info.target.0],
                )
                .map_err(|e| IndexerError::StorageError(e.to_string()))?;
            return Ok(());
        }
        let (name, is_remote) = if let Some(n) = ref_info.name.strip_prefix("refs/remotes/") {
            (n.to_string(), 1)
        } else if let Some(n) = ref_info.name.strip_prefix("refs/heads/") {
            (n.to_string(), 0)
        } else {
            (ref_info.name.clone(), 0)
        };
        self.conn
            .execute(
                "INSERT INTO branches (name, tip_oid, is_remote, is_default) VALUES (?1, ?2, ?3, 0) \
                 ON CONFLICT(name) DO UPDATE SET tip_oid = excluded.tip_oid, is_remote = excluded.is_remote",
                rusqlite::params![name, ref_info.target.0, is_remote],
            )
            .map_err(|e| IndexerError::StorageError(e.to_string()))?;
        Ok(())
    }

    fn is_commit_indexed(&self, id: &Oid) -> Result<bool, IndexerError> {
        let count: i64 = self
            .conn
            .query_row(
                "SELECT count(*) FROM commits WHERE oid = ?1",
                [&id.0],
                |row| row.get(0),
            )
            .map_err(|e| IndexerError::StorageError(e.to_string()))?;
        Ok(count > 0)
    }

    fn remove_ref(&mut self, ref_name: &str) -> Result<(), IndexerError> {
        let (name, is_tag) = if let Some(n) = ref_name.strip_prefix("refs/tags/") {
            (n.to_string(), true)
        } else if let Some(n) = ref_name.strip_prefix("refs/remotes/") {
            (n.to_string(), false)
        } else if let Some(n) = ref_name.strip_prefix("refs/heads/") {
            (n.to_string(), false)
        } else {
            (ref_name.to_string(), false)
        };
        let sql = if is_tag {
            "DELETE FROM tags WHERE name = ?1"
        } else {
            "DELETE FROM branches WHERE name = ?1 AND is_remote = ?2"
        };
        self.conn
            .execute(sql, rusqlite::params![name, 0])
            .map_err(|e| IndexerError::StorageError(e.to_string()))?;
        Ok(())
    }

    fn commit(mut self: Box<Self>) -> Result<(), IndexerError> {
        self.conn
            .execute_batch("COMMIT")
            .map_err(|e| IndexerError::StorageError(e.to_string()))?;
        self.active = false;
        Ok(())
    }

    fn rollback(mut self: Box<Self>) -> Result<(), IndexerError> {
        self.conn
            .execute_batch("ROLLBACK")
            .map_err(|e| IndexerError::StorageError(e.to_string()))?;
        self.active = false;
        Ok(())
    }
}

/// Open a connection with migrations applied, wrapped for the provider.
pub fn open_indexed(path: &std::path::Path) -> crate::Result<RefCell<rusqlite::Connection>> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let mut conn = rusqlite::Connection::open(path)?;
    crate::migrations::apply_migrations(&mut conn)?;
    Ok(RefCell::new(conn))
}
