//! `RepositoryService` (docs/04 §6): repository discovery and overview.

use crate::state::State;
use gitx_git::Repository;
use rusqlite::OptionalExtension;
use serde_json::json;

/// The default persisted index location, shared by the CLI and TUI:
/// `<git_dir>/gitx/index.sqlite` or the configured cache directory (docs/16 §6).
pub fn default_index_path(repo: &Repository) -> std::path::PathBuf {
    let config = gitx_core::Config::default();
    match config.cache_dir() {
        Some(dir) => {
            let name = repo
                .work_dir()
                .and_then(|w| w.file_name())
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "repository".into());
            std::path::PathBuf::from(dir).join(format!("{name}.sqlite"))
        }
        None => repo.git_dir().join("gitx").join("index.sqlite"),
    }
}

/// Statistics read from the persisted index (fast path, docs/13 §3).
#[derive(Debug, Clone)]
pub struct IndexedStats {
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

pub struct RepositoryService<'a> {
    pub repo: &'a Repository,
}

impl<'a> RepositoryService<'a> {
    pub fn new(repo: &'a Repository) -> Self {
        Self { repo }
    }

    /// Discover a repository from a path or the current directory.
    pub fn discover(path: Option<&std::path::Path>) -> anyhow::Result<Repository> {
        match path {
            Some(p) => Repository::discover(p)
                .map_err(|e| anyhow::anyhow!("cannot open repository at {}: {e}", p.display())),
            None => Repository::discover(".").map_err(|e| anyhow::anyhow!("{e}")),
        }
    }

    /// Current repository + index state (docs/04 §9).
    pub fn state(&self) -> State {
        State::detect(self.repo, &default_index_path(self.repo))
    }

    /// High-level overview as JSON-ish data (docs/07 `gitx info`).
    pub fn info(&self) -> serde_json::Value {
        let head = self.repo.head_commit_id().ok();
        let head_commit = head.and_then(|id| self.repo.find_commit(id).ok());
        let branches = self.repo.branches().unwrap_or_default();
        let tags = self.repo.tags().unwrap_or_default();
        let state = self.state();
        json!({
            "work_dir": self.repo.work_dir().map(|p| p.display().to_string()).unwrap_or_else(|| "<bare>".to_string()),
            "git_dir": self.repo.git_dir().display().to_string(),
            "head": head.map(|id| id.to_string()),
            "head_message": head_commit.as_ref().map(|c| c.message.clone()),
            "state": state.git,
            "index_state": state.index,
            "branches": branches.len(),
            "tags": tags.len(),
        })
    }

    /// Read repository statistics from a fresh index when possible (docs/13 §3).
    pub fn stats_from_index(&self) -> anyhow::Result<Option<IndexedStats>> {
        let path = default_index_path(self.repo);
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
        let branches: u64 =
            conn.query_row("SELECT count(*) FROM branches", [], |row| row.get(0))?;
        let tags: u64 = conn.query_row("SELECT count(*) FROM tags", [], |row| row.get(0))?;
        let first: Option<i64> = conn
            .query_row("SELECT min(timestamp) FROM commits", [], |row| row.get(0))
            .optional()?;
        let latest: i64 =
            conn.query_row("SELECT max(timestamp) FROM commits", [], |row| row.get(0))?;
        let mut languages: Vec<(String, u64)> = conn
            .prepare(
                "SELECT language, count(*) FROM files WHERE is_current = 1 GROUP BY language ORDER BY count(*) DESC",
            )?
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect::<Result<Vec<_>, _>>()?;
        languages.retain(|(lang, _)| lang != "none");
        let age_days = (latest.saturating_sub(first.unwrap_or(latest)).max(0) / 86_400) as u64;
        Ok(Some(IndexedStats {
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
}
