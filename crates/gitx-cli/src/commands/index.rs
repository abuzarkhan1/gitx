use crate::cli::{Cli, IndexAction};
use crate::commands::{open_repo, print_json};
use anyhow::Context;
use rusqlite::OptionalExtension;
use serde_json::json;
use std::path::PathBuf;

/// Default persisted index location: `<git_dir>/gitx/index.sqlite`.
///
/// The index lives inside `.git/` so it never pollutes the worktree or shows
/// up in repository analysis. When `[index] cache_dir` or `GITX_CACHE_DIR` is
/// set, the index lives in that directory instead (docs/16 §6).
pub fn default_index_path(repo: &gitx_git::Repository) -> PathBuf {
    let config = gitx_core::Config::default();
    match config.cache_dir() {
        Some(dir) => {
            let name = repo
                .work_dir()
                .and_then(|w| w.file_name())
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "repository".into());
            PathBuf::from(dir).join(format!("{name}.sqlite"))
        }
        None => repo.git_dir().join("gitx").join("index.sqlite"),
    }
}

/// Build a fully-populated GitX index (commits, parents, files, branches,
/// tags, authors) into `conn`. Used by `scan`/`refresh` (persisted) and by
/// `search` (in-memory). FTS5 search tables are populated by triggers.
pub fn build_index(
    conn: &mut rusqlite::Connection,
    repo: &gitx_git::Repository,
) -> anyhow::Result<()> {
    gitx_storage::migrations::apply_migrations(conn)?;

    let head = repo
        .head_commit_id()
        .with_context(|| "repository has no commits")?;
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
    }
    // Freshness metadata (docs/09 §5): the last-seen HEAD oid lets index
    // consumers verify the index is current, and the next refresh can detect
    // rewritten history.
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
    tx.commit()?;
    Ok(())
}

/// Whether a persisted index exists, is readable, and matches the repository's
/// current HEAD (docs/13 §3: analysis may trust a fresh index).
pub fn index_is_fresh(repo: &gitx_git::Repository) -> bool {
    let Some(head) = repo.head_commit_id().ok() else {
        return false;
    };
    let path = default_index_path(repo);
    if !path.exists() {
        return false;
    }
    let conn = match rusqlite::Connection::open(&path) {
        Ok(c) => c,
        Err(_) => return false,
    };
    // A newer-schema index is not trusted for reads (docs/18 §7).
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

/// Repository statistics read from the persisted index instead of recomputing
/// them from Git (docs/13 §3). Returns `None` when the index is missing or
/// unreadable; callers fall back to live analysis.
#[derive(Debug, Clone, serde::Serialize)]
pub struct IndexStats {
    pub commits: u64,
    pub contributors: u64,
    pub files: u64,
    pub branches: u64,
    pub tags: u64,
    pub age_days: u64,
    pub first_commit: Option<i64>,
    pub latest_commit: Option<i64>,
    pub languages: Vec<(String, u64)>,
}

pub fn stats_from_index(repo: &gitx_git::Repository) -> anyhow::Result<Option<IndexStats>> {
    let path = default_index_path(repo);
    if !path.exists() {
        return Ok(None);
    }
    let conn = rusqlite::Connection::open(&path)?;
    gitx_storage::migrations::ensure_schema_compatible(&conn)?;
    let commits: u64 = conn
        .query_row("SELECT count(*) FROM commits", [], |row| row.get(0))
        .map_err(|e| anyhow::anyhow!("index corrupt: {e}"))?;
    if commits == 0 {
        return Ok(None);
    }
    let contributors: u64 =
        conn.query_row("SELECT count(DISTINCT author_id) FROM commits", [], |row| {
            row.get(0)
        })?;
    let files: u64 = conn.query_row(
        "SELECT count(*) FROM files WHERE is_current = 1",
        [],
        |row| row.get(0),
    )?;
    let branches: u64 = conn.query_row("SELECT count(*) FROM branches", [], |row| row.get(0))?;
    let tags: u64 = conn.query_row("SELECT count(*) FROM tags", [], |row| row.get(0))?;
    let first: Option<i64> = conn
        .query_row("SELECT min(timestamp) FROM commits", [], |row| row.get(0))
        .optional()?;
    let latest: i64 = conn.query_row("SELECT max(timestamp) FROM commits", [], |row| row.get(0))?;
    let mut languages: Vec<(String, u64)> = conn
        .prepare("SELECT language, count(*) FROM files WHERE is_current = 1 GROUP BY language ORDER BY count(*) DESC")?
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect::<Result<Vec<_>, _>>()?;
    languages.retain(|(lang, _)| lang != "none");
    let age_days = (latest.saturating_sub(first.unwrap_or(latest)).max(0) / 86_400) as u64;
    Ok(Some(IndexStats {
        commits,
        contributors,
        files,
        branches,
        tags,
        age_days,
        first_commit: first,
        latest_commit: Some(latest),
        languages,
    }))
}

pub fn scan(cli: &Cli) -> anyhow::Result<()> {
    run_indexer(cli, false)
}

pub fn refresh(cli: &Cli) -> anyhow::Result<()> {
    // True incremental refresh via the gitx-index Indexer (docs/09): refs are
    // re-read, only new commits are walked, and the walk stops at already
    // indexed boundaries.
    run_indexer(cli, true)
}

/// User-visible progress on stderr (docs/09 §8); nothing pollutes stdout so
/// `--json` output stays machine-clean (docs/07 §18).
struct ConsoleProgress {
    last_report: usize,
}

impl gitx_index::ProgressReporter for ConsoleProgress {
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

/// Set by Ctrl-C so a long scan/refresh aborts cleanly (docs/09 §7).
static CANCELLED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

fn install_cancel_handler() {
    // Best-effort: if a handler was already installed, keep going.
    let _ = ctrlc::set_handler(|| {
        CANCELLED.store(true, std::sync::atomic::Ordering::SeqCst);
    });
}

fn run_indexer(cli: &Cli, incremental: bool) -> anyhow::Result<()> {
    let started = std::time::Instant::now();
    install_cancel_handler();
    CANCELLED.store(false, std::sync::atomic::Ordering::SeqCst);
    let repo = open_repo(cli)?;
    let path = default_index_path(&repo);
    tracing::info!(incremental, path = %path.display(), "indexer start");

    let conn = gitx_storage::open_indexed(&path)?;
    let storage = gitx_storage::SqliteStorageProvider::new(&conn);
    let indexer = gitx_index::Indexer::new(&repo, &storage);
    let mut progress = ConsoleProgress { last_report: 0 };

    if incremental {
        indexer.refresh_with(&mut progress, &CANCELLED)?;
    } else {
        indexer.scan_with(&mut progress, &CANCELLED)?;
    }

    // Keep derived entities fresh: tags (INSERT OR IGNORE) and reflog entries
    // (docs/09 §3 lists tag/reflog changes among incremental triggers).
    {
        let mut conn = conn.borrow_mut();
        let tx = conn.transaction()?;
        {
            let mut stmt_tag =
                tx.prepare_cached("INSERT OR IGNORE INTO tags (name, target_oid) VALUES (?1, ?2)")?;
            for tag in repo.tags()? {
                stmt_tag.execute(rusqlite::params![tag.name, tag.target.to_string()])?;
            }
            let mut stmt_reflog = tx.prepare_cached(
                "INSERT OR IGNORE INTO reflog_entries (reference, old_oid, new_oid, actor, timestamp, message) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            )?;
            for entry in gitx_analysis::collect_reflog(&repo)? {
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
    }

    let count: i64 = conn
        .borrow()
        .query_row("SELECT count(*) FROM commits", [], |row| row.get(0))
        .map_err(|e| anyhow::anyhow!("index corrupt: {e}"))?;

    // Persist the analysis cache (docs/13 §3): hotspots/risk/health then read
    // from the fresh index instead of recomputing from Git.
    if let Ok(analysis) = gitx_analysis::analyze_repository(&repo) {
        let head = repo.head_commit_id().ok().map(|h| h.to_string());
        if let Some(head) = head {
            let mut conn = conn.borrow_mut();
            let _ = gitx_analysis::cache::store(&mut conn, &repo, &analysis, &head);
        }
    }

    // Rewritten-history warning (docs/09 §5): the previous HEAD was not
    // reachable from any current ref, so the index contains commits that no
    // longer exist in the repository.
    let rewritten: String = conn
        .borrow()
        .query_row(
            "SELECT value FROM index_metadata WHERE key = 'rewritten_detected'",
            [],
            |row| row.get(0),
        )
        .unwrap_or_else(|_| "0".to_string());
    if rewritten == "1" {
        tracing::warn!("history appears rewritten — index may hold stale commits");
        if !cli.json {
            eprintln!(
                "warning: history appears rewritten (force-push/rebase) — the index may hold stale \
                 commits; run `gitx index rebuild` to rescan from scratch"
            );
        }
    }
    tracing::info!(
        commits = count,
        elapsed_ms = started.elapsed().as_millis() as u64,
        "indexer done"
    );

    if cli.json {
        return print_json(&json!({
            "index": path.display().to_string(),
            "commits": count,
            "mode": if incremental { "incremental" } else { "full" },
        }));
    }
    println!(
        "Indexed {count} commits at {} ({})",
        path.display(),
        if incremental { "incremental" } else { "full" }
    );
    Ok(())
}

pub fn index_command(cli: &Cli, action: IndexAction) -> anyhow::Result<()> {
    let repo = open_repo(cli)?;
    let path = default_index_path(&repo);

    match action {
        IndexAction::Status => {
            if !path.exists() {
                if cli.json {
                    return print_json(&json!({"index": null, "commits": 0, "exists": false}));
                }
                println!("No index at {} (run `gitx scan`)", path.display());
                return Ok(());
            }
            // Corruption detection (docs/09 §10): a read failure reports and
            // suggests a rebuild instead of silently misbehaving.
            let conn = rusqlite::Connection::open(&path)?;
            let count: Result<i64, _> =
                conn.query_row("SELECT count(*) FROM commits", [], |row| row.get(0));
            match count {
                Ok(count) => {
                    // Newer-schema detection (docs/18 §7): a valid index written
                    // by a newer build is explained, not silently trusted.
                    if let Err(e) = gitx_storage::migrations::ensure_schema_compatible(&conn) {
                        anyhow::bail!("{e}");
                    }
                    if cli.json {
                        return print_json(&json!({
                            "index": path.display().to_string(),
                            "commits": count,
                            "exists": true,
                        }));
                    }
                    println!("Index at {} — {count} commits", path.display());
                    Ok(())
                }
                Err(e) => anyhow::bail!(
                    "index corrupt: cannot read {} ({e}). Run `gitx index rebuild` to rebuild safely",
                    path.display()
                ),
            }
        }
        IndexAction::Rebuild => {
            // Atomic rebuild (docs/09 §9): build a fresh temporary index,
            // validate it, then swap it in. The old index is never destroyed
            // before the replacement is valid.
            let repo = open_repo(cli)?;
            let tmp = path.with_extension("sqlite.tmp");
            {
                let conn = gitx_storage::open_indexed(&tmp)?;
                let mut conn = conn.borrow_mut();
                build_index(&mut conn, &repo)?;
                // Validate the freshly built index before swapping.
                let count: i64 = conn
                    .query_row("SELECT count(*) FROM commits", [], |row| row.get(0))
                    .map_err(|e| anyhow::anyhow!("index corrupt: fresh build failed ({e})"))?;
                if count == 0 {
                    anyhow::bail!("index corrupt: fresh build produced no commits");
                }
                // Persist the analysis cache so hotspots/risk/health read from
                // the rebuilt index immediately (docs/13 §3).
                if let Ok(analysis) = gitx_analysis::analyze_repository(&repo)
                    && let Ok(head) = repo.head_commit_id()
                {
                    let _ = gitx_analysis::cache::store(&mut conn, &repo, &analysis, &head.to_string());
                }
            }
            // Swap (atomic on POSIX; fall back to remove+rename elsewhere).
            if path.exists() {
                let _ = std::fs::remove_file(&path);
            }
            std::fs::rename(&tmp, &path)?;
            if cli.json {
                return print_json(&json!({"index": path.display().to_string(), "rebuilt": true}));
            }
            println!("Rebuilt index at {}", path.display());
            Ok(())
        }
        IndexAction::Clear => {
            if path.exists() {
                std::fs::remove_file(&path)?;
            }
            if cli.json {
                return print_json(&json!({"index": path.display().to_string(), "cleared": true}));
            }
            println!("Cleared index at {}", path.display());
            Ok(())
        }
    }
}
