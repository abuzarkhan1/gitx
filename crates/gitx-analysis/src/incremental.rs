//! Incremental analysis cache update (docs/13 §3, docs/10).
//!
//! When an incremental index refresh brings a few new commits, recomputing
//! the whole analysis from scratch is O(history) — on a large repository the
//! first command after each commit pays a full tree-diff walk. Instead, this
//! module applies the *delta* of the new commits (a bounded walk from git,
//! mirroring the pipeline's per-commit accumulation exactly) to the persisted
//! per-file aggregates, then re-normalizes hotspot/risk/health across the
//! current file set: O(new commits) diffs + O(current files) math.
//!
//! Preconditions for an exact delta (otherwise the caller falls back to the
//! full pipeline): a stored analysis, a clean (non-rewritten) history, and
//! the previous analysis head still reachable from HEAD. File-level
//! aggregates (change count, lines, bug fixes, contributors, ownership) stay
//! bit-exact. Windowed *scoring* signals (churn, complexity) are read from
//! the persisted columns, matching the fidelity the cache path already has;
//! a full `gitx index rebuild` reconciles windowed drift.

use crate::cache::{author_id_for, health_value, load, write_health_metrics};
use crate::classification::classify_commit_message;
use crate::hotspots::{HotspotWeights, calculate_hotspot_score_with, calculate_risk_score};
use crate::pipeline::{RECENT_DAYS, compute_health_with};
use gitx_core::types::CommitClassification;
use gitx_git::Repository;
use gitx_git::models::{Commit, ObjectId};
use rusqlite::OptionalExtension;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;

/// More new commits than this → run the full pipeline instead of a giant
/// delta (bulk imports and initial backfills are better served by one full
/// analysis). Bounds the worst case of the delta walk.
const MAX_INCREMENTAL_COMMITS: usize = 200;

/// Per-file delta accumulated from the new commits, mirroring the pipeline's
/// `fold_commit` (docs/10 §3) so the incremental result equals what the full
/// walk would have produced for these commits.
#[derive(Default)]
struct Delta {
    changes: u32,
    added: u64,
    deleted: u64,
    recent_churn: u64,
    recent_changes: u32,
    bugs: u32,
    authors: HashMap<String, u64>,
    introduced: Option<i64>,
    last: Option<i64>,
}

/// Apply the new commits' delta to the persisted analysis cache. Returns
/// `Ok(true)` when applied incrementally, `Ok(false)` when the preconditions
/// are not met (caller runs the full pipeline), and propagates only
/// unexpected errors (caller treats those as a full-pipeline fallback too).
pub fn try_update_incremental(
    conn: &mut rusqlite::Connection,
    repo: &Repository,
) -> anyhow::Result<bool> {
    let computed: Option<String> = conn
        .query_row(
            "SELECT value FROM index_metadata WHERE key = 'analysis_computed'",
            [],
            |row| row.get(0),
        )
        .optional()?;
    if computed.as_deref() != Some("1") {
        return Ok(false);
    }
    let rewritten: String = conn
        .query_row(
            "SELECT value FROM index_metadata WHERE key = 'rewritten_detected'",
            [],
            |row| row.get(0),
        )
        .unwrap_or_else(|_| "0".to_string());
    if rewritten == "1" {
        return Ok(false);
    }
    let prev_head: String = match conn
        .query_row(
            "SELECT value FROM index_metadata WHERE key = 'analysis_head'",
            [],
            |row| row.get(0),
        )
        .optional()?
    {
        Some(h) => h,
        None => return Ok(false),
    };
    let head = repo.head_commit_id()?;
    let head_str = head.to_string();
    if prev_head == head_str {
        return Ok(false); // cache already fresh; nothing to do
    }

    // New commits between the analysis head and HEAD (bounded boundary walk:
    // newest-first BFS that stops at `prev_head`, exactly the pipeline's
    // `rev_walk(head)` minus `rev_walk(prev_head)`).
    let new_commits = walk_new_commits(repo, head, &prev_head)?;

    let cutoff = chrono::Utc::now().timestamp() - RECENT_DAYS * 86_400;
    let mut deltas: HashMap<PathBuf, Delta> = HashMap::new();
    let mut new_recent_changes: u64 = 0;
    for commit in &new_commits {
        let is_fix = classify_commit_message(&commit.message) == CommitClassification::Fix;
        let is_recent = commit.author.time >= cutoff;
        let parent_tree = commit
            .parents
            .first()
            .and_then(|p| repo.find_commit(*p).ok())
            .map(|p| p.tree_id);
        let changes = repo
            .diff_tree_to_tree(parent_tree, commit.tree_id)
            .unwrap_or_default();
        for c in &changes {
            if is_recent {
                new_recent_changes += 1;
            }
            let d = deltas.entry(c.path.clone()).or_default();
            d.changes += 1;
            d.added += c.insertions as u64;
            d.deleted += c.deletions as u64;
            if is_recent {
                d.recent_churn += (c.insertions + c.deletions) as u64;
                d.recent_changes += 1;
            }
            if is_fix {
                d.bugs += 1;
            }
            if d.introduced.is_none() {
                d.introduced = Some(commit.author.time);
            }
            d.last = Some(commit.author.time);
            *d.authors.entry(author_key(commit)).or_insert(0) += c.insertions as u64;
        }
    }

    let now = chrono::Utc::now().timestamp();
    let tx = conn.transaction()?;
    let new_current_files = apply_deltas(&tx, repo, &head_str, &deltas, now)?;
    recompute_scores(&tx)?;

    // Health + evidence (docs/10 §8). Recovery risk is kept from the stored
    // value: adding commits neither creates unreachable objects nor empties
    // the reflog, so it cannot change — recomputing it would re-walk the
    // whole object database (docs/13 §4).
    let Some(analysis) = load(&tx)? else {
        return Ok(false);
    };
    let total_changes: i64 = tx
        .query_row(
            "SELECT COALESCE(SUM(commit_count), 0) FROM file_metrics",
            [],
            |row| row.get(0),
        )
        .map_err(|e| anyhow::anyhow!("incremental analysis: {e}"))?;
    let recent_changes = health_value(&tx, "evidence.recent_changes").unwrap_or(0.0) as i64
        + new_recent_changes as i64;
    let mut recently_added = health_value(&tx, "evidence.recently_added").unwrap_or(0.0) as i64;
    for (path, d) in &deltas {
        // A file introduced by the new commits and still current counts as
        // recently added (windowed; aging out is reconciled by a full scan).
        let introduced = d.introduced.unwrap_or(0);
        if introduced >= cutoff && new_current_files.contains(&path.display().to_string()) {
            recently_added += 1;
        }
    }
    let current_files = analysis.files.len();
    let recovery_risk = health_value(&tx, "recovery_risk").unwrap_or(0.0);
    let health = compute_health_with(
        repo,
        &analysis.files,
        total_changes as u64,
        recent_changes as u64,
        recently_added as usize,
        current_files,
        Some(recovery_risk),
    )?;
    let commits =
        health_value(&tx, "evidence.commits").unwrap_or(0.0) as i64 + new_commits.len() as i64;
    let contributors: i64 = tx
        .query_row(
            "SELECT COUNT(DISTINCT author_id) FROM commits WHERE author_id IS NOT NULL",
            [],
            |row| row.get(0),
        )
        .map_err(|e| anyhow::anyhow!("incremental analysis: {e}"))?;
    let recent_churn: i64 = tx
        .query_row(
            "SELECT COALESCE(SUM(recent_churn), 0) FROM file_metrics",
            [],
            |row| row.get(0),
        )
        .map_err(|e| anyhow::anyhow!("incremental analysis: {e}"))?;
    write_health_metrics(
        &tx,
        &health,
        commits as u64,
        contributors as usize,
        current_files,
        current_files,
        total_changes,
        recent_changes,
        recently_added,
        recent_churn,
        now,
    )?;

    tx.execute(
        "INSERT OR REPLACE INTO index_metadata (key, value) VALUES (?1, ?2)",
        rusqlite::params!["analysis_head", head_str],
    )?;
    tx.execute(
        "INSERT OR REPLACE INTO index_metadata (key, value) VALUES (?1, ?2)",
        rusqlite::params!["analysis_computed", "1"],
    )?;
    tx.commit()?;
    tracing::info!(
        new_commits = new_commits.len(),
        touched_files = deltas.len(),
        "incremental analysis update"
    );
    Ok(true)
}

/// Newest-first BFS from `head` that stops at `prev_head` (docs/13 §3).
/// Mirrors `rev_walk(head)` minus `rev_walk(prev_head)`: every commit
/// reachable from HEAD that is not reachable from the previous analysis head.
fn walk_new_commits(
    repo: &Repository,
    head: ObjectId,
    prev_head: &str,
) -> anyhow::Result<Vec<Commit>> {
    let mut queue = VecDeque::new();
    let mut visited = HashSet::new();
    queue.push_back(head);
    let mut commits = Vec::new();
    let mut saw_boundary = false;
    while let Some(id) = queue.pop_front() {
        if id.to_string() == prev_head {
            saw_boundary = true;
            continue;
        }
        if !visited.insert(id) {
            continue;
        }
        let commit = repo.find_commit(id)?;
        for parent in &commit.parents {
            queue.push_back(*parent);
        }
        commits.push(commit);
        if commits.len() > MAX_INCREMENTAL_COMMITS {
            anyhow::bail!(
                "more than {MAX_INCREMENTAL_COMMITS} new commits since the last analysis; \
                 falling back to a full recompute"
            );
        }
    }
    if !saw_boundary {
        anyhow::bail!(
            "previous analysis head {prev_head} is no longer reachable from HEAD; \
             falling back to a full recompute"
        );
    }
    Ok(commits)
}

/// Upsert `files`/`file_metrics`/`file_ownership` rows for the touched paths.
/// Returns the paths of files created by the new commits that still exist at
/// HEAD (feed `evidence.recently_added`).
fn apply_deltas(
    tx: &rusqlite::Transaction<'_>,
    repo: &Repository,
    head_str: &str,
    deltas: &HashMap<PathBuf, Delta>,
    now: i64,
) -> anyhow::Result<HashSet<String>> {
    let mut new_current_files = HashSet::new();
    if deltas.is_empty() {
        return Ok(new_current_files);
    }
    // File ids + the set of paths present in the HEAD tree (is_current).
    let mut file_ids: HashMap<String, i64> = HashMap::new();
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
    let head_commit = repo.find_commit(repo.head_commit_id()?)?;
    let current_set: HashSet<String> = repo
        .list_blobs(head_commit.tree_id)?
        .iter()
        .map(|p| p.display().to_string())
        .collect();

    for (path, d) in deltas {
        let path_str = path.display().to_string();
        let language = path
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_else(|| "none".into());
        let is_current = current_set.contains(&path_str) as i64;
        let file_id = match file_ids.get(&path_str) {
            Some(id) => {
                tx.execute(
                    "UPDATE files SET is_current = ?1, language = ?2, last_commit_oid = ?3 \
                     WHERE id = ?4",
                    rusqlite::params![is_current, language, head_str, id],
                )?;
                *id
            }
            None => {
                tx.execute(
                    "INSERT INTO files (path, first_commit_oid, last_commit_oid, language, is_current) \
                     VALUES (?1, ?2, ?3, ?4, ?5) ON CONFLICT(path) DO NOTHING",
                    rusqlite::params![path_str.as_str(), head_str, head_str, language, is_current],
                )?;
                let id: i64 = tx.query_row(
                    "SELECT id FROM files WHERE path = ?1",
                    [path_str.as_str()],
                    |r| r.get(0),
                )?;
                file_ids.insert(path_str.clone(), id);
                if is_current == 1 {
                    new_current_files.insert(path_str.clone());
                }
                id
            }
        };

        // Ownership: merge the new commits' per-author lines into the stored
        // baselines and recompute the shares (exact — all authors are stored).
        let mut author_lines: HashMap<i64, u64> = HashMap::new();
        {
            let mut stmt =
                tx.prepare("SELECT author_id, lines FROM file_ownership WHERE file_id = ?1")?;
            let rows = stmt.query_map([file_id], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
            })?;
            for row in rows {
                let (author_id, lines) = row?;
                author_lines.insert(author_id, lines.max(0) as u64);
            }
        }
        for (author, lines) in &d.authors {
            let author_id = author_id_for(tx, author)?;
            *author_lines.entry(author_id).or_insert(0) += lines;
        }
        let total: u64 = author_lines.values().sum::<u64>().max(1);
        let mut stmt_ownership = tx.prepare(
            "INSERT INTO file_ownership (file_id, author_id, contribution_pct, lines) \
             VALUES (?1, ?2, ?3, ?4) \
             ON CONFLICT(file_id, author_id) DO UPDATE SET \
              contribution_pct = excluded.contribution_pct, \
              lines = excluded.lines",
        )?;
        for (author_id, lines) in &author_lines {
            stmt_ownership.execute(rusqlite::params![
                file_id,
                author_id,
                (*lines as f64 / total as f64) * 100.0,
                *lines as i64,
            ])?;
        }
        drop(stmt_ownership);

        // Metrics: add the delta to the stored aggregates. complexity_score
        // holds net lines (added − deleted), matching the full cache store.
        let net: i64 = d.added as i64 - d.deleted as i64;
        tx.execute(
            "INSERT INTO file_metrics \
             (file_id, commit_count, total_insertions, total_deletions, recent_churn, \
              contributor_count, bug_fix_count, complexity_score, hotspot_score, updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 0, ?9) \
             ON CONFLICT(file_id) DO UPDATE SET \
              commit_count = commit_count + excluded.commit_count, \
              total_insertions = total_insertions + excluded.total_insertions, \
              total_deletions = total_deletions + excluded.total_deletions, \
              recent_churn = recent_churn + excluded.recent_churn, \
              contributor_count = excluded.contributor_count, \
              bug_fix_count = bug_fix_count + excluded.bug_fix_count, \
              complexity_score = COALESCE(complexity_score, 0) + excluded.complexity_score, \
              updated_at = excluded.updated_at",
            rusqlite::params![
                file_id,
                d.changes,
                d.added as i64,
                d.deleted as i64,
                d.recent_churn as i64,
                author_lines.len() as i64,
                d.bugs,
                net,
                now,
            ],
        )?;
    }
    Ok(new_current_files)
}

/// Recompute hotspot/risk/classification for every current file from the
/// updated aggregates (O(current files), docs/10 §5): the scores are
/// max-normalized across the corpus, so one file's delta can shift every
/// file's normalized signal.
fn recompute_scores(tx: &rusqlite::Transaction<'_>) -> anyhow::Result<()> {
    let (max_freq, max_churn, max_bugs): (i64, i64, i64) = tx.query_row(
        "SELECT COALESCE(MAX(commit_count), 0), COALESCE(MAX(recent_churn), 0), \
                COALESCE(MAX(bug_fix_count), 0) \
         FROM file_metrics",
        [],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    )?;
    let max_complexity: f64 = tx.query_row(
        "SELECT COALESCE(MAX(complexity_score), 0) FROM file_metrics",
        [],
        |row| row.get(0),
    )?;

    // ownership concentration per file, one query for the whole corpus.
    let mut ownership: HashMap<i64, f64> = HashMap::new();
    {
        let mut stmt = tx.prepare(
            "SELECT file_id, SUM(lines), MAX(lines) FROM file_ownership GROUP BY file_id",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })?;
        for row in rows {
            let (file_id, total, max_lines) = row?;
            ownership.insert(
                file_id,
                if total > 0 {
                    max_lines as f64 / total as f64 * 100.0
                } else {
                    0.0
                },
            );
        }
    }

    let mut stmt = tx.prepare(
        "SELECT f.id, fm.commit_count, fm.recent_churn, fm.bug_fix_count, fm.complexity_score \
         FROM files f JOIN file_metrics fm ON fm.file_id = f.id WHERE f.is_current = 1",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, i64>(1)?,
            row.get::<_, i64>(2)?,
            row.get::<_, i64>(3)?,
            row.get::<_, f64>(4)?,
        ))
    })?;
    let mut stmt_hotspot = tx.prepare(
        "INSERT OR REPLACE INTO hotspots (file_id, hotspot_score, risk_score, classification, computed_at) \
         VALUES (?1, ?2, ?3, ?4, ?5)",
    )?;
    for row in rows {
        let (file_id, freq, churn, bugs, complexity) = row?;
        let n_frequency = scale(freq as u32, max_freq as u32);
        let n_churn = scale(churn as u32, max_churn as u32);
        let n_bugs = scale(bugs as u32, max_bugs as u32);
        let n_complexity = scale_u64(complexity.max(0.0) as u64, max_complexity.max(0.0) as u64);
        let concentration = ownership.get(&file_id).copied().unwrap_or(0.0);
        let hotspot = calculate_hotspot_score_with(
            HotspotWeights::default(),
            n_frequency,
            n_churn,
            n_bugs,
            concentration,
            n_complexity,
            RECENT_DAYS as u32,
        );
        let risk = calculate_risk_score(&hotspot, concentration, n_churn, n_complexity) / 4.0;
        stmt_hotspot.execute(rusqlite::params![
            file_id,
            hotspot.value,
            risk,
            hotspot.classification,
            chrono::Utc::now().timestamp(),
        ])?;
    }
    Ok(())
}

fn author_key(commit: &Commit) -> String {
    format!("{} <{}>", commit.author.name, commit.author.email)
}

fn scale(value: u32, max: u32) -> f64 {
    if max == 0 {
        0.0
    } else {
        value as f64 / max as f64 * 100.0
    }
}

fn scale_u64(value: u64, max: u64) -> f64 {
    if max == 0 {
        0.0
    } else {
        value as f64 / max as f64 * 100.0
    }
}
