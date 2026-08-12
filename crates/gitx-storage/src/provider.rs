use gitx_index::contracts::{StorageProvider, Transaction};
use gitx_index::error::IndexerError;
use gitx_index::models::{Commit, Oid, RefInfo};
use rusqlite::OptionalExtension;
use std::cell::{RefCell, RefMut};
use std::collections::HashSet;
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

    fn get_meta(&self, key: &str) -> Result<Option<String>, IndexerError> {
        let conn = self.conn.borrow();
        conn.query_row(
            "SELECT value FROM index_metadata WHERE key = ?1",
            [key],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| IndexerError::StorageError(e.to_string()))
    }

    fn get_indexed_oids(&self) -> Result<HashSet<String>, IndexerError> {
        let conn = self.conn.borrow();
        let mut stmt = conn
            .prepare("SELECT oid FROM commits")
            .map_err(|e| IndexerError::StorageError(e.to_string()))?;
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| IndexerError::StorageError(e.to_string()))?;
        let mut set = HashSet::new();
        for row in rows {
            set.insert(row.map_err(|e| IndexerError::StorageError(e.to_string()))?);
        }
        Ok(set)
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

impl SqliteTransaction<'_> {
    /// Resolve (or insert) an author row by name+email, returning its id
    /// (docs/06 authors table). Mirrors `build_index`'s author handling.
    fn author_id(&mut self, name: &str, email: &str) -> Result<i64, IndexerError> {
        let existing: Option<i64> = self
            .conn
            .query_row(
                "SELECT id FROM authors WHERE name = ?1 AND email = ?2",
                rusqlite::params![name, email],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| IndexerError::StorageError(e.to_string()))?;
        match existing {
            Some(id) => Ok(id),
            None => {
                self.conn
                    .execute(
                        "INSERT INTO authors (name, email) VALUES (?1, ?2)",
                        rusqlite::params![name, email],
                    )
                    .map_err(|e| IndexerError::StorageError(e.to_string()))?;
                Ok(self.conn.last_insert_rowid())
            }
        }
    }
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
        // Resolve (or insert) author + committer rows so the index's
        // commits.author_id/committer_id are populated exactly like the full
        // `build_index` path (docs/06): search joins, contributor stats, and
        // the FTS author triggers all read them. Missing identity falls back
        // to a deterministic "unknown" row rather than a NULL FK.
        let author_id = self.author_id(
            commit.author_name.as_deref().unwrap_or("unknown"),
            commit.author_email.as_deref().unwrap_or(""),
        )?;
        let committer_id = self.author_id(
            commit.committer_name.as_deref().unwrap_or("unknown"),
            commit.committer_email.as_deref().unwrap_or(""),
        )?;
        self.conn
            .execute(
                "INSERT INTO commits (oid, tree_oid, author_id, committer_id, timestamp, message) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6) \
                 ON CONFLICT(oid) DO UPDATE SET \
                   tree_oid = excluded.tree_oid, \
                   author_id = excluded.author_id, \
                   committer_id = excluded.committer_id, \
                   timestamp = excluded.timestamp, \
                   message = excluded.message",
                rusqlite::params![
                    commit.id.0,
                    commit.tree_id.as_deref().unwrap_or(""),
                    author_id,
                    committer_id,
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

    fn write_meta(&mut self, key: &str, value: &str) -> Result<(), IndexerError> {
        self.conn
            .execute(
                "INSERT INTO index_metadata (key, value) VALUES (?1, ?2) \
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                rusqlite::params![key, value],
            )
            .map_err(|e| IndexerError::StorageError(e.to_string()))?;
        Ok(())
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
    // Newer-schema detection (docs/18 §7): refuse to silently read or
    // overwrite an index written by a newer gitx build.
    crate::migrations::ensure_schema_compatible(&conn)?;
    crate::migrations::apply_migrations(&mut conn)?;
    Ok(RefCell::new(conn))
}
