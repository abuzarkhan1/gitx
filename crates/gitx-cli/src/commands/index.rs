use crate::cli::{Cli, IndexAction};
use crate::commands::{open_repo, print_json};
use anyhow::Context;
use rusqlite::OptionalExtension;
use serde_json::json;
use std::path::PathBuf;

/// Default persisted index location: `<git_dir>/gitx/index.sqlite`.
///
/// The index lives inside `.git/` so it never pollutes the worktree or shows
/// up in repository analysis (docs/16 §6 cache location).
pub fn default_index_path(repo: &gitx_git::Repository) -> PathBuf {
    repo.git_dir().join("gitx").join("index.sqlite")
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
    tx.commit()?;
    Ok(())
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

fn run_indexer(cli: &Cli, incremental: bool) -> anyhow::Result<()> {
    let repo = open_repo(cli)?;
    let path = default_index_path(&repo);

    let conn = gitx_storage::open_indexed(&path)?;
    let storage = gitx_storage::SqliteStorageProvider::new(&conn);
    let indexer = gitx_index::Indexer::new(&repo, &storage);
    let mut progress = gitx_index::NoopProgress;

    if incremental {
        indexer.refresh(&mut progress)?;
    } else {
        indexer.scan(&mut progress)?;
    }

    let count: i64 = conn
        .borrow()
        .query_row("SELECT count(*) FROM commits", [], |row| row.get(0))?;

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
            let conn = rusqlite::Connection::open(&path)?;
            let count: i64 =
                conn.query_row("SELECT count(*) FROM commits", [], |row| row.get(0))?;
            if cli.json {
                return print_json(
                    &json!({"index": path.display().to_string(), "commits": count, "exists": true}),
                );
            }
            println!("Index at {} — {count} commits", path.display());
            Ok(())
        }
        IndexAction::Rebuild => refresh(cli),
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
