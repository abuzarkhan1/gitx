//! `IndexService` (docs/04 §6): scan/refresh/status/rebuild/clear over the
//! persisted SQLite index, including the analysis cache (docs/13 §3).

use crate::repository::default_index_path;
use gitx_git::Repository;
use gitx_index::ProgressReporter;
use rusqlite::OptionalExtension;
use std::cell::RefCell;
use std::sync::atomic::AtomicBool;

/// Console progress reporting to stderr (docs/09 §8); stdout stays clean.
#[derive(Default)]
pub struct ConsoleProgress {
    last_report: usize,
}

impl ProgressReporter for ConsoleProgress {
    fn report(&mut self, progress: &gitx_index::Progress) {
        if progress.commits_processed.saturating_sub(self.last_report) >= 1000 {
            self.last_report = progress.commits_processed;
            eprintln!(
                "indexing: {} commits processed…",
                progress.commits_processed
            );
        }
    }
}

pub struct IndexService<'a> {
    pub repo: &'a Repository,
}

impl<'a> IndexService<'a> {
    pub fn new(repo: &'a Repository) -> Self {
        Self { repo }
    }

    pub fn index_path(&self) -> std::path::PathBuf {
        default_index_path(self.repo)
    }

    /// Full scan or incremental refresh (docs/09). Returns the commit count.
    /// The analysis cache is persisted afterwards so hotspots/risk/health read
    /// from the index (docs/13 §3).
    pub fn scan(&self, incremental: bool) -> anyhow::Result<u64> {
        self.scan_with(incremental, &AtomicBool::new(false))
    }

    pub fn scan_with(&self, incremental: bool, cancelled: &AtomicBool) -> anyhow::Result<u64> {
        let path = self.index_path();
        let conn = gitx_storage::open_indexed(&path)?;
        let storage = gitx_storage::SqliteStorageProvider::new(&conn);
        let indexer = gitx_index::Indexer::new(self.repo, &storage);
        let mut progress = ConsoleProgress::default();
        if incremental {
            indexer.refresh_with(&mut progress, cancelled)?;
        } else {
            indexer.scan_with(&mut progress, cancelled)?;
        }
        self.refresh_derived(&conn)?;

        let count: i64 = conn
            .borrow()
            .query_row("SELECT count(*) FROM commits", [], |row| row.get(0))
            .map_err(|e| anyhow::anyhow!("index corrupt: {e}"))?;

        // Persist the analysis cache (docs/13 §3).
        if let Ok(analysis) = gitx_analysis::analyze_repository(self.repo)
            && let Ok(head) = self.repo.head_commit_id()
        {
            let mut conn = conn.borrow_mut();
            let _ = gitx_analysis::cache::store(&mut conn, self.repo, &analysis, &head.to_string());
        }
        Ok(count as u64)
    }

    /// Tags + reflog upserts after a scan/refresh (docs/09 §3).
    fn refresh_derived(&self, conn: &RefCell<rusqlite::Connection>) -> anyhow::Result<()> {
        let mut conn = conn.borrow_mut();
        let tx = conn.transaction()?;
        {
            let mut stmt_tag =
                tx.prepare_cached("INSERT OR IGNORE INTO tags (name, target_oid) VALUES (?1, ?2)")?;
            for tag in self.repo.tags()? {
                stmt_tag.execute(rusqlite::params![tag.name, tag.target.to_string()])?;
            }
            let mut stmt_reflog = tx.prepare_cached(
                "INSERT OR IGNORE INTO reflog_entries (reference, old_oid, new_oid, actor, timestamp, message) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            )?;
            for entry in gitx_analysis::collect_reflog(self.repo)? {
                stmt_reflog.execute(rusqlite::params![
                    entry.reference,
                    entry.previous_oid.to_string(),
                    entry.new_oid.to_string(),
                    entry.actor_name,
                    entry.timestamp,
                    entry.message,
                ])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    /// Whether the persisted index exists and matches HEAD (docs/13 §3).
    pub fn is_fresh(&self) -> bool {
        let Some(head) = self.repo.head_commit_id().ok() else {
            return false;
        };
        let path = self.index_path();
        if !path.exists() {
            return false;
        }
        let conn = match rusqlite::Connection::open(&path) {
            Ok(c) => c,
            Err(_) => return false,
        };
        if gitx_storage::migrations::ensure_schema_compatible(&conn).is_err() {
            return false;
        }
        let stored: Option<String> = conn
            .query_row(
                "SELECT value FROM index_metadata WHERE key = 'last_head'",
                [],
                |row| row.get(0),
            )
            .ok();
        stored.as_deref() == Some(head.to_string().as_str())
    }

    /// Incremental refresh when the persisted index is stale or absent
    /// (docs/16 `[index] auto_refresh`): cheap when only HEAD moved, a full
    /// build on first run. No-op when fresh. Used by the CLI and TUI so the
    /// index builds itself instead of every analytical command recomputing
    /// live from Git (docs/13 §3 sub-second reads).
    pub fn refresh_if_stale(&self) -> anyhow::Result<()> {
        if self.is_fresh() {
            return Ok(());
        }
        let cancelled = AtomicBool::new(false);
        self.scan_with(true, &cancelled)?;
        Ok(())
    }

    /// Commit count from the index; `Err` for a corrupt/newer-schema index
    /// (docs/09 §10, docs/18 §7 — surfaced as exit code 5 by the CLI).
    pub fn status(&self) -> anyhow::Result<u64> {
        let path = self.index_path();
        let conn = rusqlite::Connection::open(&path)?;
        let count: i64 = conn
            .query_row("SELECT count(*) FROM commits", [], |row| row.get(0))
            .map_err(|e| anyhow::anyhow!("index corrupt: {e}"))?;
        gitx_storage::migrations::ensure_schema_compatible(&conn)?;
        Ok(count as u64)
    }

    /// Atomic rebuild: temp-index → validate → swap (docs/09 §9).
    pub fn rebuild(&self) -> anyhow::Result<()> {
        let path = self.index_path();
        let tmp = path.with_extension("sqlite.tmp");
        {
            let conn = gitx_storage::open_indexed(&tmp)?;
            let mut conn = conn.borrow_mut();
            build_index(&mut conn, self.repo)?;
            let count: i64 = conn
                .query_row("SELECT count(*) FROM commits", [], |row| row.get(0))
                .map_err(|e| anyhow::anyhow!("index corrupt: fresh build failed ({e})"))?;
            if count == 0 {
                anyhow::bail!("index corrupt: fresh build produced no commits");
            }
            if let Ok(analysis) = gitx_analysis::analyze_repository(self.repo)
                && let Ok(head) = self.repo.head_commit_id()
            {
                let _ =
                    gitx_analysis::cache::store(&mut conn, self.repo, &analysis, &head.to_string());
            }
        }
        if path.exists() {
            let _ = std::fs::remove_file(&path);
        }
        std::fs::rename(&tmp, &path)?;
        Ok(())
    }

    pub fn clear(&self) -> anyhow::Result<()> {
        let path = self.index_path();
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
        Ok(())
    }
}

/// Build a fully-populated GitX index (commits, parents, files, branches,
/// tags, authors) into `conn`. Used by `rebuild` (persisted) and by `gitx
/// search` (in-memory). FTS5 search tables are populated by triggers.
pub fn build_index(conn: &mut rusqlite::Connection, repo: &Repository) -> anyhow::Result<()> {
    gitx_storage::migrations::apply_migrations(conn)?;

    let head = repo
        .head_commit_id()
        .map_err(|e| anyhow::anyhow!("repository has no commits: {e}"))?;
    let head_commit = repo.find_commit(head)?;

    let tx = conn.transaction()?;
    {
        let mut author_ids: std::collections::HashMap<(String, String), i64> =
            std::collections::HashMap::new();
        let mut get_author =
            |name: &str, email: &str, tx: &rusqlite::Transaction<'_>| -> rusqlite::Result<i64> {
                let key = (name.to_string(), email.to_string());
                if let Some(id) = author_ids.get(&key) {
                    return Ok(*id);
                }
                let existing: Option<i64> = tx
                    .query_row(
                        "SELECT id FROM authors WHERE name = ?1 AND email = ?2",
                        rusqlite::params![name, email],
                        |row| row.get(0),
                    )
                    .optional()?;
                let id = match existing {
                    Some(id) => id,
                    None => {
                        tx.execute(
                            "INSERT INTO authors (name, email) VALUES (?1, ?2)",
                            rusqlite::params![name, email],
                        )?;
                        tx.last_insert_rowid()
                    }
                };
                author_ids.insert(key, id);
                Ok(id)
            };

        let mut stmt_commit = tx.prepare_cached(
            "INSERT OR IGNORE INTO commits (oid, author_id, committer_id, tree_oid, timestamp, message) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        )?;
        let mut stmt_parent = tx.prepare_cached(
            "INSERT OR IGNORE INTO commit_parents (commit_oid, parent_oid, parent_index) VALUES (?1, ?2, ?3)",
        )?;

        for id_res in repo.rev_walk(head)? {
            let commit = repo.find_commit(id_res?)?;
            let author_id = get_author(&commit.author.name, &commit.author.email, &tx)?;
            let committer_id = get_author(&commit.committer.name, &commit.committer.email, &tx)?;
            stmt_commit.execute(rusqlite::params![
                commit.id.to_string(),
                author_id,
                committer_id,
                commit.tree_id.to_string(),
                commit.author.time,
                commit.message,
            ])?;
            for (idx, parent) in commit.parents.iter().enumerate() {
                stmt_parent.execute(rusqlite::params![
                    commit.id.to_string(),
                    parent.to_string(),
                    idx as i64
                ])?;
            }
        }

        // Files present in HEAD.
        let mut stmt_file = tx.prepare_cached(
            "INSERT OR IGNORE INTO files (path, first_commit_oid, last_commit_oid, language, is_current) \
             VALUES (?1, ?2, ?3, ?4, 1)",
        )?;
        for path in repo.list_blobs(head_commit.tree_id)? {
            let ext = path
                .extension()
                .map(|e| e.to_string_lossy().to_lowercase())
                .unwrap_or_else(|| "none".to_string());
            stmt_file.execute(rusqlite::params![
                path.display().to_string(),
                head.to_string(),
                head.to_string(),
                ext,
            ])?;
        }

        // Branches and tags.
        let mut stmt_branch = tx.prepare_cached(
            "INSERT OR IGNORE INTO branches (name, tip_oid, is_remote, is_default) VALUES (?1, ?2, ?3, 0)",
        )?;
        for branch in repo.branches()? {
            stmt_branch.execute(rusqlite::params![
                branch.name,
                branch.target.to_string(),
                branch.is_remote,
            ])?;
        }
        let mut stmt_tag =
            tx.prepare_cached("INSERT OR IGNORE INTO tags (name, target_oid) VALUES (?1, ?2)")?;
        for tag in repo.tags()? {
            stmt_tag.execute(rusqlite::params![tag.name, tag.target.to_string()])?;
        }

        // Symbols from heuristic source extraction (docs/11 §2, docs/21 Stage
        // 6): populated so `gitx search --symbols` works against the index.
        {
            let mut file_ids: std::collections::HashMap<String, i64> =
                std::collections::HashMap::new();
            {
                let mut stmt = tx.prepare("SELECT id, path FROM files")?;
                let rows = stmt.query_map([], |row| {
                    Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
                })?;
                for row in rows {
                    let (id, path) = row?;
                    file_ids.insert(path, id);
                }
            }
            tx.execute("DELETE FROM symbols", [])?;
            let mut stmt_symbol = tx.prepare_cached(
                "INSERT INTO symbols (file_id, name, kind, line) VALUES (?1, ?2, ?3, ?4)",
            )?;
            for (path, syms) in
                gitx_analysis::symbols::extract_symbols_from_tree(repo, head_commit.tree_id)?
            {
                let Some(file_id) = file_ids.get(&path.display().to_string()) else {
                    continue;
                };
                for s in syms.iter().take(400) {
                    stmt_symbol.execute(rusqlite::params![
                        file_id,
                        s.name,
                        s.kind,
                        s.line as i64,
                    ])?;
                }
            }
        }

        // Freshness metadata (docs/09 §5).
        tx.execute(
            "INSERT INTO index_metadata (key, value) VALUES (?1, ?2) \
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            rusqlite::params!["last_head", head.to_string()],
        )?;
        tx.execute(
            "INSERT INTO index_metadata (key, value) VALUES (?1, ?2) \
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            rusqlite::params!["rewritten_detected", "0"],
        )?;
    }
    tx.commit()?;
    Ok(())
}
