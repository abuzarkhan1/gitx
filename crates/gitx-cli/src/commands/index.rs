use crate::cli::{Cli, IndexAction};
use crate::commands::{open_repo, print_json};
use gitx_services::IndexService;
use serde_json::json;

/// Default persisted index location (docs/16 §6) — thin delegate so callers
/// in this crate keep a single import point.
pub fn default_index_path(repo: &gitx_git::Repository) -> std::path::PathBuf {
    gitx_services::repository::default_index_path(repo)
}

/// Build a fully-populated index into `conn` (used by `search` for its
/// in-memory index and by rebuild). Delegates to the service layer.
pub fn build_index(
    conn: &mut rusqlite::Connection,
    repo: &gitx_git::Repository,
) -> anyhow::Result<()> {
    gitx_services::index::build_index(conn, repo)
}

/// Whether a persisted index exists, is readable, and matches the repository's
/// current HEAD (docs/13 §3).
pub fn index_is_fresh(repo: &gitx_git::Repository) -> bool {
    IndexService::new(repo).is_fresh()
}

/// Repository statistics read from the persisted index (docs/13 §3).
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
    Ok(gitx_services::RepositoryService::new(repo)
        .stats_from_index()?
        .map(|s| IndexStats {
            commits: s.commits,
            contributors: s.contributors,
            files: s.files,
            branches: s.branches,
            tags: s.tags,
            age_days: s.age_days,
            first_commit: s.first_commit,
            latest_commit: s.latest_commit,
            languages: s.languages,
        }))
}

pub fn scan(cli: &Cli) -> anyhow::Result<()> {
    run_indexer(cli, false)
}

pub fn refresh(cli: &Cli) -> anyhow::Result<()> {
    run_indexer(cli, true)
}

/// Set by Ctrl-C so a long scan/refresh aborts cleanly (docs/09 §7).
static CANCELLED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

fn install_cancel_handler() {
    let _ = ctrlc::set_handler(|| {
        CANCELLED.store(true, std::sync::atomic::Ordering::SeqCst);
    });
}

fn run_indexer(cli: &Cli, incremental: bool) -> anyhow::Result<()> {
    let started = std::time::Instant::now();
    install_cancel_handler();
    CANCELLED.store(false, std::sync::atomic::Ordering::SeqCst);
    let repo = open_repo(cli)?;
    // `[index] enabled = false` (docs/16 §3): scan/refresh are a deliberate
    // no-op so nothing is written, and analysis stays live.
    let config = crate::commands::config::load_config_for(cli, &repo)?;
    if !config.index.enabled {
        println!("indexing is disabled ([index] enabled = false); skipping");
        return Ok(());
    }
    let service = IndexService::new(&repo);
    let path = service.index_path();
    tracing::info!(incremental, path = %path.display(), "indexer start");

    let count = service.scan_with(incremental, &CANCELLED)?;

    // Rewritten-history warning (docs/09 §5): surfaced from index metadata.
    {
        let conn = rusqlite::Connection::open(&path)?;
        let rewritten: String = conn
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
    let service = IndexService::new(&repo);
    let path = service.index_path();

    match action {
        IndexAction::Status => {
            if !path.exists() {
                if cli.json {
                    return print_json(&json!({"index": null, "commits": 0, "exists": false}));
                }
                println!("No index at {} (run `gitx scan`)", path.display());
                return Ok(());
            }
            match service.status() {
                Ok(count) => {
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
                Err(e) => anyhow::bail!("{e:#}. Run `gitx index rebuild` to rebuild safely"),
            }
        }
        IndexAction::Rebuild => {
            service.rebuild()?;
            if cli.json {
                return print_json(&json!({"index": path.display().to_string(), "rebuilt": true}));
            }
            println!("Rebuilt index at {}", path.display());
            Ok(())
        }
        IndexAction::Clear => {
            service.clear()?;
            if cli.json {
                return print_json(&json!({"index": path.display().to_string(), "cleared": true}));
            }
            println!("Cleared index at {}", path.display());
            Ok(())
        }
    }
}
