//! Incremental analysis cache (docs/13 §3): a refresh after new commits must
//! apply the delta of those commits to the persisted per-file aggregates —
//! exactly matching what a full recompute would store — instead of re-walking
//! history.

use gitx_analysis::analyze_repository;
use gitx_git::Repository;
use gitx_services::IndexService;

#[path = "../common/mod.rs"]
mod common;
use common::sample_repo;

fn metrics_for(conn: &rusqlite::Connection, path: &str) -> Option<(i64, i64, i64, i64, i64)> {
    conn.query_row(
        "SELECT fm.commit_count, fm.total_insertions, fm.total_deletions, \
                fm.bug_fix_count, fm.contributor_count \
         FROM file_metrics fm JOIN files f ON f.id = fm.file_id WHERE f.path = ?1",
        [path],
        |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
            ))
        },
    )
    .ok()
}

#[test]
fn incremental_refresh_keeps_file_metrics_exact() {
    let Some(repo) = sample_repo() else {
        eprintln!("skipping: git CLI unavailable");
        return;
    };
    let git = Repository::discover(repo.path()).expect("open fixture");
    let service = IndexService::new(&git);
    service.scan(false).expect("full scan");

    // One new commit with a known delta: an extra line in src/lib.rs.
    repo.write(
        "src/lib.rs",
        "pub fn hello() { println!(\"hi\"); }\npub fn world() {}\npub fn extra() {}\n",
    );
    repo.commit("feat: extra fn");
    service.scan(true).expect("incremental refresh");

    let conn = rusqlite::Connection::open(service.index_path()).expect("open index");
    let (commits, added, deleted, bugs, contributors) =
        metrics_for(&conn, "src/lib.rs").expect("src/lib.rs metrics");
    assert_eq!(commits, 3, "src/lib.rs touched by 3 commits");
    assert_eq!(added, 4, "1 initial + 2 fix + 1 new line");
    assert_eq!(deleted, 1, "the rewritten line from the fix commit");
    assert_eq!(bugs, 1, "only the 'fix:' commit counts");
    assert_eq!(contributors, 1, "single author");

    // Untouched files must be byte-identical to the full scan.
    assert_eq!(
        metrics_for(&conn, "README.md").map(|m| m.0),
        Some(1),
        "README.md touched once"
    );
    assert_eq!(
        metrics_for(&conn, "main.rs").map(|m| m.0),
        Some(1),
        "main.rs touched once"
    );

    // Cache points at HEAD.
    let head = git.head_commit_id().expect("head").to_string();
    let stored: String = conn
        .query_row(
            "SELECT value FROM index_metadata WHERE key = 'analysis_head'",
            [],
            |row| row.get(0),
        )
        .expect("analysis_head");
    assert_eq!(stored, head, "analysis cache must track HEAD");

    // The incremental numbers must equal a fresh full analysis, file for file
    // (the exactness guarantee of the delta).
    let live = analyze_repository(&git).expect("live analysis");
    for f in live.files.iter() {
        let path = f.path.display().to_string();
        let (commits, added, deleted, bugs, contributors) =
            metrics_for(&conn, &path).expect("metrics row");
        assert_eq!(
            commits, f.metrics.change_frequency as i64,
            "{path}: commit_count"
        );
        assert_eq!(added, f.metrics.lines_added as i64, "{path}: insertions");
        assert_eq!(deleted, f.metrics.lines_deleted as i64, "{path}: deletions");
        assert_eq!(bugs, f.metrics.bug_fix_count as i64, "{path}: bug fixes");
        assert_eq!(
            contributors, f.metrics.unique_contributors as i64,
            "{path}: contributors"
        );
    }

    // Health evidence is present and recomputed for the new state.
    let overall: f64 = conn
        .query_row(
            "SELECT metric_value FROM metrics WHERE metric_key = 'overall'",
            [],
            |row| row.get(0),
        )
        .expect("overall health");
    assert!(
        (0.0..=100.0).contains(&overall),
        "health score sane: {overall}"
    );
}

#[test]
fn incremental_refresh_marks_deleted_files_non_current() {
    let Some(repo) = sample_repo() else {
        eprintln!("skipping: git CLI unavailable");
        return;
    };
    let git = Repository::discover(repo.path()).expect("open fixture");
    let service = IndexService::new(&git);
    service.scan(false).expect("full scan");

    std::fs::remove_file(repo.path().join("main.rs")).expect("remove main.rs");
    repo.commit("chore: drop main.rs");
    service.scan(true).expect("incremental refresh");

    let conn = rusqlite::Connection::open(service.index_path()).expect("open index");
    let is_current: i64 = conn
        .query_row(
            "SELECT is_current FROM files WHERE path = 'main.rs'",
            [],
            |row| row.get(0),
        )
        .expect("main.rs row");
    assert_eq!(is_current, 0, "deleted file must be flagged non-current");
    let current: i64 = conn
        .query_row(
            "SELECT count(*) FROM files WHERE is_current = 1",
            [],
            |row| row.get(0),
        )
        .expect("current count");
    assert_eq!(current, 2, "src/lib.rs + README.md remain current");
}

#[test]
fn incremental_refresh_updates_ownership_lines() {
    let Some(repo) = sample_repo() else {
        eprintln!("skipping: git CLI unavailable");
        return;
    };
    let git = Repository::discover(repo.path()).expect("open fixture");
    let service = IndexService::new(&git);
    service.scan(false).expect("full scan");

    repo.write(
        "src/lib.rs",
        "pub fn hello() { println!(\"hi\"); }\npub fn world() {}\npub fn extra() {}\n",
    );
    repo.commit("feat: extra fn");
    service.scan(true).expect("incremental refresh");

    let conn = rusqlite::Connection::open(service.index_path()).expect("open index");
    let lines: i64 = conn
        .query_row(
            "SELECT SUM(o.lines) FROM file_ownership o JOIN files f ON f.id = o.file_id \
             WHERE f.path = 'src/lib.rs'",
            [],
            |row| row.get(0),
        )
        .expect("ownership lines");
    assert_eq!(lines, 4, "author lines accumulate to total insertions");
    let pct: f64 = conn
        .query_row(
            "SELECT MAX(o.contribution_pct) FROM file_ownership o JOIN files f ON f.id = o.file_id \
             WHERE f.path = 'src/lib.rs'",
            [],
            |row| row.get(0),
        )
        .expect("ownership share");
    assert_eq!(pct, 100.0, "single author owns the file");
}

#[test]
fn incremental_falls_back_to_full_analysis_without_cache() {
    let Some(repo) = sample_repo() else {
        eprintln!("skipping: git CLI unavailable");
        return;
    };
    let git = Repository::discover(repo.path()).expect("open fixture");
    let service = IndexService::new(&git);
    service.scan(false).expect("full scan");

    // Wipe the cached-analysis metadata; the next refresh must still converge
    // to a fresh cache via the full-pipeline fallback.
    {
        let conn = rusqlite::Connection::open(service.index_path()).expect("open index");
        conn.execute(
            "DELETE FROM index_metadata WHERE key IN ('analysis_computed', 'analysis_head')",
            [],
        )
        .expect("clear analysis meta");
    }
    repo.write("README.md", "# Sample\n\nMore docs.\n");
    repo.commit("docs: expand readme");
    service.scan(true).expect("incremental refresh");

    let conn = rusqlite::Connection::open(service.index_path()).expect("open index");
    let computed: String = conn
        .query_row(
            "SELECT value FROM index_metadata WHERE key = 'analysis_computed'",
            [],
            |row| row.get(0),
        )
        .expect("analysis_computed");
    assert_eq!(computed, "1", "fallback must store a fresh analysis cache");
    let (commits, added, _, _, _) = metrics_for(&conn, "README.md").expect("README metrics");
    assert_eq!(
        commits, 2,
        "README.md touched by both the scan and the new commit"
    );
    assert_eq!(added, 3, "1 line initial + 2 added by the fallback path");
}
