use crate::error::Result;
use rusqlite::Connection;

/// Mirrored on disk in `migrations/0001_initial.sql` — keep both in sync.
const SCHEMA_V1: &str = r#"
CREATE TABLE index_metadata (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE repositories (
    id INTEGER PRIMARY KEY,
    root_path TEXT NOT NULL UNIQUE,
    git_dir TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE authors (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT,
    normalized_name TEXT,
    normalized_email TEXT
);

CREATE TABLE commits (
    oid TEXT PRIMARY KEY,
    author_id INTEGER,
    committer_id INTEGER,
    tree_oid TEXT,
    timestamp INTEGER NOT NULL,
    message TEXT NOT NULL,
    FOREIGN KEY(author_id) REFERENCES authors(id),
    FOREIGN KEY(committer_id) REFERENCES authors(id)
);

CREATE TABLE commit_parents (
    commit_oid TEXT NOT NULL,
    parent_oid TEXT NOT NULL,
    parent_index INTEGER NOT NULL,
    PRIMARY KEY(commit_oid, parent_oid),
    FOREIGN KEY(commit_oid) REFERENCES commits(oid)
);

CREATE TABLE files (
    id INTEGER PRIMARY KEY,
    path TEXT NOT NULL UNIQUE,
    first_commit_oid TEXT,
    last_commit_oid TEXT,
    language TEXT,
    is_current INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE file_changes (
    id INTEGER PRIMARY KEY,
    commit_oid TEXT NOT NULL,
    file_id INTEGER NOT NULL,
    change_type TEXT NOT NULL,
    old_path TEXT,
    new_path TEXT,
    insertions INTEGER NOT NULL DEFAULT 0,
    deletions INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY(commit_oid) REFERENCES commits(oid),
    FOREIGN KEY(file_id) REFERENCES files(id)
);

CREATE TABLE branches (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    tip_oid TEXT,
    is_remote INTEGER NOT NULL DEFAULT 0,
    is_default INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE tags (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    target_oid TEXT NOT NULL
);

CREATE TABLE reflog_entries (
    id INTEGER PRIMARY KEY,
    reference TEXT NOT NULL,
    old_oid TEXT NOT NULL,
    new_oid TEXT NOT NULL,
    actor TEXT,
    timestamp INTEGER,
    message TEXT
);

CREATE TABLE file_metrics (
    file_id INTEGER PRIMARY KEY,
    commit_count INTEGER NOT NULL,
    total_insertions INTEGER NOT NULL,
    total_deletions INTEGER NOT NULL,
    recent_churn INTEGER NOT NULL,
    contributor_count INTEGER NOT NULL,
    bug_fix_count INTEGER NOT NULL,
    complexity_score REAL,
    hotspot_score REAL,
    updated_at INTEGER NOT NULL
);

INSERT INTO index_metadata (key, value) VALUES ('schema_version', '1');
CREATE INDEX idx_commits_author ON commits(author_id);
CREATE INDEX idx_commits_committer ON commits(committer_id);
CREATE INDEX idx_commit_parents_parent ON commit_parents(parent_oid);
CREATE INDEX idx_file_changes_commit ON file_changes(commit_oid);
CREATE INDEX idx_file_changes_file ON file_changes(file_id);
"#;

/// Mirrored on disk in `migrations/0002_search_and_derived.sql` — keep both in sync.
const SCHEMA_V2: &str = r#"
-- Full-text search index (SQLite FTS5). These virtual tables back the
-- gitx-search crate; the search_index from docs/06 is realized here.
CREATE VIRTUAL TABLE commits_fts USING fts5(
    oid UNINDEXED,
    message,
    tokenize = 'porter unicode61'
);
CREATE VIRTUAL TABLE files_fts USING fts5(
    path,
    tokenize = 'porter unicode61'
);
CREATE VIRTUAL TABLE authors_fts USING fts5(
    name,
    email,
    tokenize = 'porter unicode61'
);
CREATE VIRTUAL TABLE branches_fts USING fts5(
    name,
    tokenize = 'porter unicode61'
);
CREATE VIRTUAL TABLE tags_fts USING fts5(
    name,
    tokenize = 'porter unicode61'
);

-- Keep the FTS index in sync with the underlying tables.
CREATE TRIGGER commits_ai AFTER INSERT ON commits BEGIN
    INSERT INTO commits_fts(rowid, oid, message) VALUES (new.rowid, new.oid, new.message);
END;
CREATE TRIGGER commits_ad AFTER DELETE ON commits BEGIN
    DELETE FROM commits_fts WHERE rowid = old.rowid;
END;
CREATE TRIGGER commits_au AFTER UPDATE OF message ON commits BEGIN
    DELETE FROM commits_fts WHERE rowid = old.rowid;
    INSERT INTO commits_fts(rowid, oid, message) VALUES (new.rowid, new.oid, new.message);
END;

CREATE TRIGGER files_ai AFTER INSERT ON files BEGIN
    INSERT INTO files_fts(rowid, path) VALUES (new.id, new.path);
END;
CREATE TRIGGER files_ad AFTER DELETE ON files BEGIN
    DELETE FROM files_fts WHERE rowid = old.id;
END;
CREATE TRIGGER files_au AFTER UPDATE OF path ON files BEGIN
    DELETE FROM files_fts WHERE rowid = old.id;
    INSERT INTO files_fts(rowid, path) VALUES (new.id, new.path);
END;

CREATE TRIGGER authors_ai AFTER INSERT ON authors BEGIN
    INSERT INTO authors_fts(rowid, name, email) VALUES (new.id, new.name, new.email);
END;
CREATE TRIGGER authors_ad AFTER DELETE ON authors BEGIN
    DELETE FROM authors_fts WHERE rowid = old.id;
END;

CREATE TRIGGER branches_ai AFTER INSERT ON branches BEGIN
    INSERT INTO branches_fts(rowid, name) VALUES (new.id, new.name);
END;
CREATE TRIGGER branches_ad AFTER DELETE ON branches BEGIN
    DELETE FROM branches_fts WHERE rowid = old.id;
END;

CREATE TRIGGER tags_ai AFTER INSERT ON tags BEGIN
    INSERT INTO tags_fts(rowid, name) VALUES (new.id, new.name);
END;
CREATE TRIGGER tags_ad AFTER DELETE ON tags BEGIN
    DELETE FROM tags_fts WHERE rowid = old.id;
END;

-- Derived tables (extended repository intelligence, docs/06).
CREATE TABLE file_renames (
    id INTEGER PRIMARY KEY,
    file_id INTEGER NOT NULL,
    commit_oid TEXT NOT NULL,
    old_path TEXT NOT NULL,
    new_path TEXT NOT NULL,
    FOREIGN KEY(file_id) REFERENCES files(id),
    FOREIGN KEY(commit_oid) REFERENCES commits(oid)
);

CREATE TABLE branch_commits (
    branch_id INTEGER NOT NULL,
    commit_oid TEXT NOT NULL,
    PRIMARY KEY(branch_id, commit_oid),
    FOREIGN KEY(branch_id) REFERENCES branches(id),
    FOREIGN KEY(commit_oid) REFERENCES commits(oid)
);

CREATE TABLE file_ownership (
    file_id INTEGER NOT NULL,
    author_id INTEGER NOT NULL,
    contribution_pct REAL NOT NULL DEFAULT 0,
    PRIMARY KEY(file_id, author_id),
    FOREIGN KEY(file_id) REFERENCES files(id),
    FOREIGN KEY(author_id) REFERENCES authors(id)
);

CREATE TABLE symbols (
    id INTEGER PRIMARY KEY,
    file_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    kind TEXT NOT NULL,
    line INTEGER,
    FOREIGN KEY(file_id) REFERENCES files(id)
);

CREATE TABLE dependencies (
    id INTEGER PRIMARY KEY,
    file_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    version TEXT,
    FOREIGN KEY(file_id) REFERENCES files(id)
);

CREATE TABLE dependency_events (
    id INTEGER PRIMARY KEY,
    commit_oid TEXT NOT NULL,
    dependency_name TEXT NOT NULL,
    event_type TEXT NOT NULL,
    version TEXT,
    FOREIGN KEY(commit_oid) REFERENCES commits(oid)
);

CREATE TABLE hotspots (
    file_id INTEGER PRIMARY KEY,
    hotspot_score REAL NOT NULL,
    risk_score REAL NOT NULL,
    classification TEXT NOT NULL,
    computed_at INTEGER NOT NULL,
    FOREIGN KEY(file_id) REFERENCES files(id)
);

CREATE TABLE metrics (
    id INTEGER PRIMARY KEY,
    scope TEXT NOT NULL,
    scope_id INTEGER,
    metric_key TEXT NOT NULL,
    metric_value REAL NOT NULL,
    computed_at INTEGER NOT NULL
);

CREATE INDEX idx_file_renames_file ON file_renames(file_id);
CREATE INDEX idx_branch_commits_commit ON branch_commits(commit_oid);
CREATE INDEX idx_file_ownership_file ON file_ownership(file_id);
CREATE INDEX idx_symbols_file ON symbols(file_id);
CREATE INDEX idx_dependencies_file ON dependencies(file_id);
CREATE INDEX idx_dependency_events_commit ON dependency_events(commit_oid);
CREATE INDEX idx_metrics_scope ON metrics(scope, scope_id);
UPDATE index_metadata SET value = '2' WHERE key = 'schema_version';
"#;

/// Mirrored on disk in `migrations/0003_fts_delete_triggers.sql` — keep both in sync.
///
/// v3 fixes a bug in the v2 FTS5 sync triggers: the FTS5 `'delete'` special
/// INSERT command is only valid for contentless/external-content tables and
/// raised "SQL logic error" on our normal-content tables. The corrected
/// triggers use a plain `DELETE FROM <fts> WHERE rowid = ...`. Existing v2
/// databases keep the broken triggers (the version guard skips re-running
/// SCHEMA_V2), so this migration drops and recreates them.
const SCHEMA_V3: &str = r#"
DROP TRIGGER IF EXISTS commits_ad;
DROP TRIGGER IF EXISTS commits_au;
DROP TRIGGER IF EXISTS files_ad;
DROP TRIGGER IF EXISTS files_au;
DROP TRIGGER IF EXISTS authors_ad;
DROP TRIGGER IF EXISTS branches_ad;
DROP TRIGGER IF EXISTS tags_ad;

CREATE TRIGGER commits_ad AFTER DELETE ON commits BEGIN
    DELETE FROM commits_fts WHERE rowid = old.rowid;
END;
CREATE TRIGGER commits_au AFTER UPDATE OF message ON commits BEGIN
    DELETE FROM commits_fts WHERE rowid = old.rowid;
    INSERT INTO commits_fts(rowid, oid, message) VALUES (new.rowid, new.oid, new.message);
END;
CREATE TRIGGER files_ad AFTER DELETE ON files BEGIN
    DELETE FROM files_fts WHERE rowid = old.id;
END;
CREATE TRIGGER files_au AFTER UPDATE OF path ON files BEGIN
    DELETE FROM files_fts WHERE rowid = old.id;
    INSERT INTO files_fts(rowid, path) VALUES (new.id, new.path);
END;
CREATE TRIGGER authors_ad AFTER DELETE ON authors BEGIN
    DELETE FROM authors_fts WHERE rowid = old.id;
END;
CREATE TRIGGER branches_ad AFTER DELETE ON branches BEGIN
    DELETE FROM branches_fts WHERE rowid = old.id;
END;
CREATE TRIGGER tags_ad AFTER DELETE ON tags BEGIN
    DELETE FROM tags_fts WHERE rowid = old.id;
END;

UPDATE index_metadata SET value = '3' WHERE key = 'schema_version';
"#;

/// Mirrored on disk in `migrations/0004_incremental_analysis.sql` — keep both
/// in sync.
///
/// v4 supports the incremental analysis cache (docs/13 §3): `file_ownership`
/// gains an absolute per-author line count so a refresh can apply the delta
/// of new commits to ownership shares instead of recomputing the whole
/// analysis. Rows written before v4 keep a default of 0 and are only read
/// when an incremental update touches that file, where the missing baseline
/// is resolved by the next full scan.
const SCHEMA_V4: &str = r#"
ALTER TABLE file_ownership ADD COLUMN lines INTEGER NOT NULL DEFAULT 0;

UPDATE index_metadata SET value = '4' WHERE key = 'schema_version';
"#;

/// The newest schema version this build understands (docs/18 §7: indexes
/// written by a *newer* build must be detected and explained, not silently
/// read or overwritten).
pub const CURRENT_SCHEMA_VERSION: i64 = 4;

/// Read the stored schema version without applying migrations. `None` when
/// the metadata table is missing (a v0/empty database).
pub fn stored_schema_version(conn: &Connection) -> crate::Result<Option<i64>> {
    match conn.query_row(
        "SELECT value FROM index_metadata WHERE key = 'schema_version'",
        [],
        |row| {
            let v: String = row.get(0)?;
            Ok(v.parse::<i64>().unwrap_or(0))
        },
    ) {
        Ok(v) => Ok(Some(v)),
        // A brand-new (empty) database has no metadata table yet — that is a
        // v0 database, fully compatible and awaiting migration.
        Err(rusqlite::Error::SqliteFailure(_, Some(msg))) if msg.contains("no such table") => {
            Ok(None)
        }
        Err(e) => Err(e.into()),
    }
}

/// Error if the stored schema version is newer than this build understands
/// (docs/18 §7). Returns the stored version otherwise.
pub fn ensure_schema_compatible(conn: &Connection) -> crate::Result<i64> {
    let version = stored_schema_version(conn)?.unwrap_or(0);
    if version > CURRENT_SCHEMA_VERSION {
        return Err(crate::error::Error::SchemaNewer {
            stored: version,
            supported: CURRENT_SCHEMA_VERSION,
        });
    }
    Ok(version)
}

pub fn apply_migrations(conn: &mut Connection) -> Result<()> {
    let tx = conn.transaction()?;

    let version: i64 = tx
        .query_row(
            "SELECT value FROM index_metadata WHERE key = 'schema_version'",
            [],
            |row| {
                let v: String = row.get(0)?;
                Ok(v.parse::<i64>().unwrap_or(0))
            },
        )
        .unwrap_or(0);

    if version < 1 {
        tx.execute_batch(SCHEMA_V1)?;
    }

    if version < 2 {
        tx.execute_batch(SCHEMA_V2)?;
    }

    if version < 3 {
        tx.execute_batch(SCHEMA_V3)?;
    }

    if version < 4 {
        tx.execute_batch(SCHEMA_V4)?;
    }

    tx.commit()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn open() -> crate::Connection {
        crate::Connection::open_in_memory().expect("in-memory db")
    }

    #[test]
    fn migrations_apply_and_are_idempotent() {
        let mut conn = open();
        // Applying again must not fail (schema_version guard).
        apply_migrations(&mut conn.inner).expect("second apply");
        apply_migrations(&mut conn.inner).expect("third apply");

        let version: String = conn
            .inner
            .query_row(
                "SELECT value FROM index_metadata WHERE key = 'schema_version'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(version, "4");
    }

    #[test]
    fn v1_tables_exist() {
        let conn = open();
        for table in [
            "index_metadata",
            "repositories",
            "authors",
            "commits",
            "commit_parents",
            "files",
            "file_changes",
            "branches",
            "tags",
            "reflog_entries",
            "file_metrics",
        ] {
            let count: i64 = conn
                .inner
                .query_row(
                    "SELECT count(*) FROM sqlite_master WHERE type='table' AND name=?1",
                    [table],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(count, 1, "missing v1 table: {table}");
        }
    }

    #[test]
    fn v2_tables_exist() {
        let conn = open();
        for table in [
            "file_renames",
            "branch_commits",
            "file_ownership",
            "symbols",
            "dependencies",
            "dependency_events",
            "hotspots",
            "metrics",
        ] {
            let count: i64 = conn
                .inner
                .query_row(
                    "SELECT count(*) FROM sqlite_master WHERE type='table' AND name=?1",
                    [table],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(count, 1, "missing v2 table: {table}");
        }
    }

    #[test]
    fn fts_tables_exist_and_accept_queries() {
        let conn = open();
        for table in [
            "commits_fts",
            "files_fts",
            "authors_fts",
            "branches_fts",
            "tags_fts",
        ] {
            let count: i64 = conn
                .inner
                .query_row(
                    "SELECT count(*) FROM sqlite_master WHERE type='table' AND name=?1",
                    [table],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(count, 1, "missing fts table: {table}");
        }
        // The exact queries used by gitx-search must run.
        conn.inner.execute(
            "INSERT INTO commits (oid, timestamp, message) VALUES ('abc123', 1, 'fix workspace bug')",
            [],
        )
        .unwrap();
        let n: i64 = conn
            .inner
            .query_row(
                "SELECT count(*) FROM commits_fts WHERE commits_fts MATCH 'workspace'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(n, 1, "FTS5 should index new commits via trigger");
    }

    #[test]
    fn fts_delete_and_update_triggers_keep_index_in_sync() {
        // Regression: the FTS5 'delete' special INSERT command is only valid
        // for contentless/external-content tables; on a normal content table
        // it raised "SQL logic error". Delete/update triggers must use a plain
        // DELETE FROM <fts> WHERE rowid = ... instead.
        let mut conn = open();
        apply_migrations(&mut conn.inner).expect("migrate");

        conn.inner
            .execute_batch(
                "INSERT INTO commits (oid, timestamp, message) VALUES
                    ('aaa111', 1, 'fix workspace bug'),
                    ('bbb222', 2, 'add widget feature');",
            )
            .unwrap();

        let count: i64 = conn
            .inner
            .query_row("SELECT count(*) FROM commits_fts", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 2, "insert trigger must index rows");

        // DELETE must not error and must remove the FTS row.
        conn.inner
            .execute("DELETE FROM commits WHERE oid = 'aaa111'", [])
            .unwrap();
        let count: i64 = conn
            .inner
            .query_row("SELECT count(*) FROM commits_fts", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1, "delete trigger must remove FTS row");

        // UPDATE must replace the indexed text (old text gone, new text found).
        conn.inner
            .execute(
                "UPDATE commits SET message = 'fix workspace regression' WHERE oid = 'bbb222'",
                [],
            )
            .unwrap();
        let old: i64 = conn
            .inner
            .query_row(
                "SELECT count(*) FROM commits_fts WHERE commits_fts MATCH 'widget'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(old, 0, "update trigger must drop old indexed text");
        let new: i64 = conn
            .inner
            .query_row(
                "SELECT count(*) FROM commits_fts WHERE commits_fts MATCH 'regression'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(new, 1, "update trigger must index new text");
    }

    #[test]
    fn v3_fixes_broken_v2_delete_triggers() {
        // Simulate a v2 database whose delete trigger used the FTS5 'delete'
        // special command (invalid for normal-content tables). SCHEMA_V2 in
        // this crate already carries the corrected triggers, so we recreate
        // the buggy trigger explicitly to reproduce the historical state.
        let mut conn = rusqlite::Connection::open_in_memory().unwrap();
        {
            let tx = conn.transaction().unwrap();
            tx.execute_batch(SCHEMA_V1).unwrap();
            tx.execute_batch(SCHEMA_V2).unwrap();
            tx.execute_batch(
                "DROP TRIGGER commits_ad;
                 CREATE TRIGGER commits_ad AFTER DELETE ON commits BEGIN
                     INSERT INTO commits_fts(commits_fts, rowid, oid, message)
                     VALUES ('delete', old.rowid, old.oid, old.message);
                 END;",
            )
            .unwrap();
            tx.commit().unwrap();
        }
        conn.execute(
            "INSERT INTO commits (oid, timestamp, message) VALUES ('deadbeef', 2, 'legacy commit')",
            [],
        )
        .unwrap();

        // Sanity: the buggy trigger really fails on delete.
        let buggy = conn
            .execute("DELETE FROM commits WHERE oid = 'deadbeef'", [])
            .err()
            .map(|e| e.to_string())
            .unwrap_or_default();
        assert!(
            buggy.contains("SQL logic error"),
            "v2 delete trigger should fail, got: {buggy}"
        );

        // The row survives (statement-level rollback) — migrate to v3.
        apply_migrations(&mut conn).unwrap();

        let version: String = conn
            .query_row(
                "SELECT value FROM index_metadata WHERE key = 'schema_version'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(version, "4");

        // The corrected trigger must delete without error.
        conn.execute("DELETE FROM commits WHERE oid = 'deadbeef'", [])
            .expect("v3 delete trigger must work");
        let n: i64 = conn
            .query_row("SELECT count(*) FROM commits_fts", [], |row| row.get(0))
            .unwrap();
        assert_eq!(n, 0, "FTS row must be removed by the v3 trigger");
    }

    #[test]
    fn migration_from_v1_to_v2_preserves_data() {
        // Simulate a v1 database by applying v1 only.
        let mut conn = rusqlite::Connection::open_in_memory().unwrap();
        {
            let tx = conn.transaction().unwrap();
            tx.execute_batch(SCHEMA_V1).unwrap();
            tx.commit().unwrap();
        }
        conn.execute(
            "INSERT INTO commits (oid, timestamp, message) VALUES ('deadbeef', 2, 'legacy commit')",
            [],
        )
        .unwrap();

        apply_migrations(&mut conn).unwrap();

        let version: String = conn
            .query_row(
                "SELECT value FROM index_metadata WHERE key = 'schema_version'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(version, "4");
        let n: i64 = conn
            .query_row(
                "SELECT count(*) FROM commits_fts WHERE commits_fts MATCH 'legacy'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(
            n, 0,
            "pre-existing v1 rows are not retro-indexed (FTS starts empty)"
        );
        let n: i64 = conn
            .query_row(
                "SELECT count(*) FROM commits WHERE oid='deadbeef'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(n, 1, "v1 data must survive the v2 migration");
    }
}
