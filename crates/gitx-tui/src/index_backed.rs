//! Index-backed repository statistics for the TUI overview (docs/13 §3).
//!
//! The TUI cannot depend on `gitx-cli` (that would be a cycle), so this small
//! module mirrors the CLI's `stats_from_index` query. With a fresh persisted
//! index the Overview panel starts in milliseconds; without one it falls back
//! to live Git analysis (see `app.rs`).

/// Statistics read from the persisted SQLite index.
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

/// The default persisted index location, mirroring the CLI: `<git_dir>/gitx/index.sqlite`
/// or the configured cache directory (docs/16 §6).
pub fn default_index_path(repo: &gitx_git::Repository) -> std::path::PathBuf {
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

/// Read statistics from the persisted index. Returns `None` when the index is
/// missing or unreadable (callers fall back to live computation).
pub fn stats_from_index(repo: &gitx_git::Repository) -> anyhow::Result<Option<IndexStats>> {
    let path = default_index_path(repo);
    if !path.exists() {
        return Ok(None);
    }
    let conn = rusqlite::Connection::open(&path)?;
    // Newer-schema indexes are not trusted (docs/18 §7); fall back to live.
    if gitx_storage::migrations::ensure_schema_compatible(&conn).is_err() {
        return Ok(None);
    }
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
        .ok();
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

/// Author → files map for the Contributors view, read from the persisted
/// `file_ownership` table (top-3 owners per file, docs/06). The cached
/// `RepoAnalysis` deliberately does not carry `author_lines`, so the live
/// enrichment path is empty from a fresh index; this mirrors the same query
/// the live path computes (docs/08 Contributors: files touched + top areas).
/// Returns `None` when the index is missing or has no ownership rows (callers
/// fall back to the live analysis map).
pub fn author_files_from_index(
    repo: &gitx_git::Repository,
) -> anyhow::Result<Option<std::collections::HashMap<String, Vec<String>>>> {
    let path = default_index_path(repo);
    if !path.exists() {
        return Ok(None);
    }
    let conn = rusqlite::Connection::open(&path)?;
    if gitx_storage::migrations::ensure_schema_compatible(&conn).is_err() {
        return Ok(None);
    }
    let mut map: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    let mut stmt = conn.prepare(
        "SELECT a.name, a.email, f.path FROM file_ownership o \
         JOIN authors a ON a.id = o.author_id \
         JOIN files f ON f.id = o.file_id",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
        ))
    })?;
    for row in rows {
        let (name, email, path) = row?;
        let key = format!("{name} <{email}>");
        map.entry(key).or_default().push(path);
    }
    if map.is_empty() {
        return Ok(None);
    }
    Ok(Some(map))
}
