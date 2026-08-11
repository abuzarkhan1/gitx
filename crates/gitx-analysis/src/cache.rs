//! Persisted analysis cache (docs/13 §3, docs/23 "Persistent Index" column).
//!
//! After a full scan/refresh the repository analysis (per-file metrics,
//! hotspot/risk scores, ownership concentration, composite health) is written
//! into the index's `file_metrics`, `hotspots`, `file_ownership` and `metrics`
//! tables. Commands and the TUI then read results back from a *fresh* index
//! (HEAD matches `analysis_head` metadata) instead of recomputing from Git,
//! falling back to live analysis when the cache is stale or absent.
//!
//! The cache is a write-through convenience, not a second source of truth:
//! every number stored here was computed deterministically by the pipeline
//! from Git data.

use crate::pipeline::{FileAnalysis, RepoAnalysis};
use gitx_git::Repository;
use rusqlite::{Connection, OptionalExtension};
use std::path::PathBuf;

const RECENT_CHURN_KEY: &str = "recent_churn";

/// Store `analysis` for the repository at its current HEAD into the index.
/// Idempotent (upserts); safe to call after every scan/refresh. The `files`
/// table is owned here (the incremental Indexer does not populate it): every
/// analyzed path is upserted and flagged current iff it exists in HEAD.
pub fn store(
    conn: &mut Connection,
    repo: &Repository,
    analysis: &RepoAnalysis,
    head_oid: &str,
) -> anyhow::Result<()> {
    // Files present in the HEAD tree (for the is_current flag).
    let current: std::collections::HashSet<String> = repo
        .head_commit_id()
        .ok()
        .and_then(|id| repo.find_commit(id).ok())
        .and_then(|commit| repo.list_blobs(commit.tree_id).ok())
        .map(|paths| {
            paths
                .iter()
                .map(|p| p.display().to_string())
                .collect::<std::collections::HashSet<_>>()
        })
        .unwrap_or_default();

    let tx = conn.transaction()?;
    let now = chrono::Utc::now().timestamp();
    let mut file_ids: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    {
        let mut stmt = tx.prepare("SELECT id, path FROM files")?;
        let rows = stmt.query_map([], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)))?;
        for row in rows {
            let (id, path) = row?;
            file_ids.insert(path, id);
        }
    }
    // Reset the current flag; it is recomputed below from HEAD.
    tx.execute("UPDATE files SET is_current = 0", [])?;

    let mut stmt_metrics = tx.prepare(
        "INSERT INTO file_metrics \
         (file_id, commit_count, total_insertions, total_deletions, recent_churn, \
          contributor_count, bug_fix_count, complexity_score, hotspot_score, updated_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10) \
         ON CONFLICT(file_id) DO UPDATE SET \
          commit_count = excluded.commit_count, \
          total_insertions = excluded.total_insertions, \
          total_deletions = excluded.total_deletions, \
          recent_churn = excluded.recent_churn, \
          contributor_count = excluded.contributor_count, \
          bug_fix_count = excluded.bug_fix_count, \
          complexity_score = excluded.complexity_score, \
          hotspot_score = excluded.hotspot_score, \
          updated_at = excluded.updated_at",
    )?;
    let mut stmt_hotspots = tx.prepare(
        "INSERT INTO hotspots (file_id, hotspot_score, risk_score, classification, computed_at) \
         VALUES (?1, ?2, ?3, ?4, ?5) \
         ON CONFLICT(file_id) DO UPDATE SET \
          hotspot_score = excluded.hotspot_score, \
          risk_score = excluded.risk_score, \
          classification = excluded.classification, \
          computed_at = excluded.computed_at",
    )?;
    let mut stmt_ownership = tx.prepare(
        "INSERT INTO file_ownership (file_id, author_id, contribution_pct) \
         VALUES (?1, ?2, ?3) \
         ON CONFLICT(file_id, author_id) DO UPDATE SET contribution_pct = excluded.contribution_pct",
    )?;
    let mut stmt_meta =
        tx.prepare("INSERT OR REPLACE INTO index_metadata (key, value) VALUES (?1, ?2)")?;
    let mut stmt_metric = tx.prepare(
        "INSERT OR REPLACE INTO metrics (scope, scope_id, metric_key, metric_value, computed_at) \
         VALUES (?1, ?2, ?3, ?4, ?5)",
    )?;

    for f in &analysis.files {
        let path_str = f.path.display().to_string();
        let is_current = current.contains(&path_str);
        let file_id = match file_ids.get(&path_str) {
            Some(id) => {
                tx.execute(
                    "UPDATE files SET is_current = ?1 WHERE id = ?2",
                    rusqlite::params![is_current, id],
                )?;
                *id
            }
            // File not yet in the index (deleted from HEAD, or the Indexer
            // never populated files): insert with the correct current flag.
            None => {
                tx.execute(
                    "INSERT INTO files (path, first_commit_oid, last_commit_oid, language, is_current) \
                     VALUES (?1, NULL, NULL, 'none', ?2) ON CONFLICT(path) DO NOTHING",
                    rusqlite::params![path_str, is_current],
                )?;
                let id: i64 = tx.query_row(
                    "SELECT id FROM files WHERE path = ?1",
                    [path_str.as_str()],
                    |r| r.get(0),
                )?;
                file_ids.insert(path_str, id);
                id
            }
        };
        let churn = f.metrics.lines_added as i64 + f.metrics.lines_deleted as i64;
        stmt_metrics.execute(rusqlite::params![
            file_id,
            f.metrics.change_frequency,
            f.metrics.lines_added,
            f.metrics.lines_deleted,
            churn,
            f.metrics.unique_contributors,
            f.metrics.bug_fix_count,
            f.metrics.lines_added.saturating_sub(f.metrics.lines_deleted),
            f.hotspot,
            now,
        ])?;
        stmt_hotspots.execute(rusqlite::params![
            file_id,
            f.hotspot,
            f.risk,
            f.classification,
            now,
        ])?;
        // Top-3 owners by contribution share (docs/06 file_ownership).
        let mut owners: Vec<(&String, &u64)> = f.author_lines.iter().collect();
        owners.sort_by_key(|(_, lines)| std::cmp::Reverse(**lines));
        let total: u64 = f.author_lines.values().sum::<u64>().max(1);
        for (author, lines) in owners.iter().take(3) {
            let author_id = author_id_for(&tx, author)?;
            stmt_ownership.execute(rusqlite::params![
                file_id,
                author_id,
                (**lines as f64 / total as f64) * 100.0,
            ])?;
        }
    }

    let h = &analysis.health;
    let health_metrics = [
        ("overall", h.overall_score),
        ("code_hotspots", h.code_hotspots_score),
        ("ownership_risk", h.ownership_risk_score),
        ("branch_hygiene", h.branch_hygiene_score),
        ("change_volatility", h.change_volatility_score),
        ("architecture_stability", h.architecture_stability_score),
        ("recovery_risk", h.recovery_risk_score),
        ("evidence.commits", analysis.total_commits as f64),
        ("evidence.contributors", analysis.total_contributors as f64),
        ("evidence.current_files", analysis.current_files as f64),
        ("evidence.analyzed_files", analysis.files.len() as f64),
    ];
    for (key, value) in health_metrics {
        stmt_metric.execute(rusqlite::params!["health", 0, key, value, now])?;
    }
    stmt_metric.execute(rusqlite::params![
        "health",
        0,
        RECENT_CHURN_KEY,
        analysis
            .files
            .iter()
            .map(|f| f.metrics.lines_added as i64 + f.metrics.lines_deleted as i64)
            .sum::<i64>(),
        now,
    ])?;

    stmt_meta.execute(rusqlite::params!["analysis_head", head_oid])?;
    stmt_meta.execute(rusqlite::params!["analysis_computed", "1"])?;
    drop(stmt_metrics);
    drop(stmt_hotspots);
    drop(stmt_ownership);
    drop(stmt_meta);
    drop(stmt_metric);
    tx.commit()?;
    Ok(())
}

/// Whether the index holds analysis results matching the repository's HEAD.
pub fn is_fresh(conn: &Connection, repo: &Repository) -> bool {
    let Some(head) = repo.head_commit_id().ok() else {
        return false;
    };
    let stored: Option<String> = conn
        .query_row(
            "SELECT value FROM index_metadata WHERE key = 'analysis_head'",
            [],
            |row| row.get(0),
        )
        .ok();
    stored.as_deref() == Some(head.to_string().as_str())
}

/// Load a full [`RepoAnalysis`] back from the index. Returns `None` when the
/// cache is missing or unreadable; callers fall back to live analysis.
/// `author_lines` is not persisted (only top-3 ownership shares are), so the
/// reconstructed analysis carries ownership concentration but no per-line map.
pub fn load(conn: &Connection) -> anyhow::Result<Option<RepoAnalysis>> {
    let computed: Option<String> = conn
        .query_row(
            "SELECT value FROM index_metadata WHERE key = 'analysis_computed'",
            [],
            |row| row.get(0),
        )
        .optional()?;
    if computed.as_deref() != Some("1") {
        return Ok(None);
    }

    let mut files = Vec::new();
    {
        let mut stmt = conn.prepare(
            "SELECT f.path, fm.commit_count, fm.total_insertions, fm.total_deletions, \
                    fm.recent_churn, fm.contributor_count, fm.bug_fix_count, \
                    fm.complexity_score, h.hotspot_score, h.risk_score, h.classification, \
                    COALESCE((SELECT MAX(contribution_pct) FROM file_ownership o WHERE o.file_id = f.id), 0) \
             FROM files f \
             JOIN file_metrics fm ON fm.file_id = f.id \
             JOIN hotspots h ON h.file_id = f.id \
             WHERE f.is_current = 1",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, i64>(5)?,
                row.get::<_, i64>(6)?,
                row.get::<_, f64>(7)?,
                row.get::<_, f64>(8)?,
                row.get::<_, f64>(9)?,
                row.get::<_, String>(10)?,
                row.get::<_, f64>(11)?,
            ))
        })?;
        for row in rows {
            let (
                path,
                commits,
                added,
                deleted,
                _churn,
                contributors,
                bugs,
                _complexity,
                hotspot,
                risk,
                classification,
                ownership,
            ) = row?;
            files.push(FileAnalysis {
                path: PathBuf::from(path),
                metrics: crate::metrics::FileMetrics {
                    change_frequency: commits as u32,
                    lines_added: added as u32,
                    lines_deleted: deleted as u32,
                    first_introduced: None,
                    last_modified: None,
                    bug_fix_count: bugs as u32,
                    unique_contributors: contributors as u32,
                },
                author_lines: std::collections::HashMap::new(),
                ownership_concentration: ownership,
                hotspot,
                classification: match classification.as_str() {
                    "CRITICAL" => "CRITICAL",
                    "HIGH" => "HIGH",
                    "MEDIUM" => "MEDIUM",
                    _ => "LOW",
                },
                risk,
            });
        }
    }
    files.sort_by(|a, b| {
        b.hotspot
            .partial_cmp(&a.hotspot)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let health = load_health(conn)?;

    Ok(Some(RepoAnalysis {
        files,
        total_commits: health_value(conn, "evidence.commits").unwrap_or(0.0) as u64,
        total_contributors: health_value(conn, "evidence.contributors").unwrap_or(0.0) as usize,
        current_files: health_value(conn, "evidence.current_files").unwrap_or(0.0) as usize,
        health,
        analysis_duration_ms: 0,
    }))
}

fn load_health(conn: &Connection) -> anyhow::Result<crate::health::RepoHealth> {
    Ok(crate::health::RepoHealth {
        overall_score: health_value(conn, "overall").unwrap_or(0.0),
        code_hotspots_score: health_value(conn, "code_hotspots").unwrap_or(0.0),
        ownership_risk_score: health_value(conn, "ownership_risk").unwrap_or(0.0),
        branch_hygiene_score: health_value(conn, "branch_hygiene").unwrap_or(0.0),
        change_volatility_score: health_value(conn, "change_volatility").unwrap_or(0.0),
        architecture_stability_score: health_value(conn, "architecture_stability").unwrap_or(0.0),
        recovery_risk_score: health_value(conn, "recovery_risk").unwrap_or(0.0),
    })
}

fn health_value(conn: &Connection, key: &str) -> Option<f64> {
    conn.query_row(
        "SELECT metric_value FROM metrics WHERE scope = 'health' AND metric_key = ?1",
        [key],
        |row| row.get(0),
    )
    .ok()
}

/// Resolve (or insert) an author by `Name <email>` key. Author rows are keyed
/// by name+email in the index schema.
fn author_id_for(tx: &rusqlite::Transaction<'_>, key: &str) -> anyhow::Result<i64> {
    let (name, email) = match key.split_once(" <") {
        Some((n, rest)) => (n.to_string(), rest.trim_end_matches('>').to_string()),
        None => (key.to_string(), String::new()),
    };
    let existing: Option<i64> = tx
        .query_row(
            "SELECT id FROM authors WHERE name = ?1 AND email = ?2",
            rusqlite::params![name, email],
            |row| row.get(0),
        )
        .optional()?;
    match existing {
        Some(id) => Ok(id),
        None => {
            tx.execute(
                "INSERT INTO authors (name, email) VALUES (?1, ?2)",
                rusqlite::params![name, email],
            )?;
            Ok(tx.last_insert_rowid())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn author_key_split_round_trips() {
        let (name, email) = match "Abuzar <a@x.co>".split_once(" <") {
            Some((n, rest)) => (n.to_string(), rest.trim_end_matches('>').to_string()),
            None => unreachable!(),
        };
        assert_eq!(name, "Abuzar");
        assert_eq!(email, "a@x.co");
    }
}
