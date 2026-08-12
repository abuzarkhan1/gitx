//! Exercises the gitx-index Indexer end-to-end with real providers
//! (docs/09: full scan + incremental refresh).

use gitx_git::Repository;
use gitx_index::contracts::{GitProvider, StorageProvider};
use gitx_index::{Indexer, NoopProgress};
use gitx_storage::{open_indexed, SqliteStorageProvider};
use std::cell::RefCell;

#[path = "../common/mod.rs"]
mod common;
use common::sample_repo;

fn commit_count(conn: &RefCell<rusqlite::Connection>) -> i64 {
    conn.borrow()
        .query_row("SELECT count(*) FROM commits", [], |row| row.get(0))
        .expect("count commits")
}

#[test]
fn scan_then_incremental_refresh() {
    let Some(repo) = sample_repo() else {
        eprintln!("skipping: git CLI unavailable");
        return;
    };
    let gix = Repository::discover(repo.path()).expect("open fixture");

    let conn = open_indexed(std::path::Path::new(":memory:")).expect("in-memory index");
    let storage = SqliteStorageProvider::new(&conn);
    let indexer = Indexer::new(&gix, &storage);
    let mut progress = NoopProgress;

    // Full scan indexes all 3 commits.
    indexer.scan(&mut progress).expect("scan");
    assert_eq!(commit_count(&conn), 3);

    // Add a commit; refresh picks it up incrementally.
    repo.write("notes.txt", "extra\n");
    repo.commit("docs: add notes");
    indexer.refresh(&mut progress).expect("refresh");
    assert_eq!(commit_count(&conn), 4);

    // Refresh with no changes is a no-op.
    indexer.refresh(&mut progress).expect("refresh again");
    assert_eq!(commit_count(&conn), 4);

    // Branches were written with fully-qualified names.
    let branches: i64 = conn
        .borrow()
        .query_row("SELECT count(*) FROM branches", [], |row| row.get(0))
        .expect("count branches");
    assert_eq!(branches, 1);
    let name: String = conn
        .borrow()
        .query_row("SELECT name FROM branches LIMIT 1", [], |row| row.get(0))
        .expect("branch name");
    assert_eq!(name, "main");
}

#[test]
fn refresh_stops_at_indexed_boundaries() {
    let Some(repo) = sample_repo() else {
        eprintln!("skipping: git CLI unavailable");
        return;
    };
    let gix = Repository::discover(repo.path()).expect("open fixture");

    let conn = open_indexed(std::path::Path::new(":memory:")).expect("in-memory index");
    let storage = SqliteStorageProvider::new(&conn);
    let indexer = Indexer::new(&gix, &storage);
    let mut progress = NoopProgress;

    indexer.scan(&mut progress).expect("scan");
    assert_eq!(commit_count(&conn), 3);

    // Deleting the whole table simulates a partial/corrupt index; refresh must
    // still converge to the full history without failing. Children first to
    // satisfy the foreign keys.
    conn.borrow()
        .execute_batch("DELETE FROM commit_parents; DELETE FROM file_changes; DELETE FROM commits;")
        .expect("clear commits");
    indexer.refresh(&mut progress).expect("refresh after wipe");
    assert_eq!(commit_count(&conn), 3);
}

#[test]
fn walk_stops_at_indexed_boundaries() {
    // The provider's boundary-stop walk (docs/13 §6) is what keeps refresh
    // O(new commits): a fully indexed history must yield nothing, and one new
    // commit on top must yield exactly that commit — never the indexed past.
    let Some(repo) = sample_repo() else {
        eprintln!("skipping: git CLI unavailable");
        return;
    };
    let gix = Repository::discover(repo.path()).expect("open fixture");

    let conn = open_indexed(std::path::Path::new(":memory:")).expect("in-memory index");
    let storage = SqliteStorageProvider::new(&conn);
    let indexer = Indexer::new(&gix, &storage);
    indexer.scan(&mut NoopProgress).expect("scan");

    // Every tip is indexed: the walk must not visit a single commit.
    let indexed = storage.get_indexed_oids().expect("indexed oids");
    let tips: Vec<_> = gix
        .read_refs()
        .expect("refs")
        .into_iter()
        .map(|r| r.target)
        .collect();
    let walked = gix
        .walk_commits(&tips, &indexed)
        .expect("walk")
        .collect::<Result<Vec<_>, _>>()
        .expect("collect");
    assert_eq!(walked.len(), 0, "fully indexed history must yield nothing");

    // One new commit on top: exactly it is walked, not the 3-commit history.
    repo.write("notes.txt", "extra\n");
    repo.commit("docs: add notes");
    let gix = Repository::discover(repo.path()).expect("reopen fixture");
    let tips: Vec<_> = gix
        .read_refs()
        .expect("refs")
        .into_iter()
        .map(|r| r.target)
        .collect();
    let walked = gix
        .walk_commits(&tips, &indexed)
        .expect("walk")
        .collect::<Result<Vec<_>, _>>()
        .expect("collect");
    assert_eq!(walked.len(), 1, "exactly the new commit is walked");
}

#[test]
fn scan_populates_authors_tree_and_languages() {
    // Regression (product-hardening audit): the incremental Indexer used to
    // write commits with NULL author_id/committer_id/tree_oid and the analysis
    // cache inserted files with language 'none', so stats/search/TUI showed
    // "contributors 0" and an empty language breakdown from a fresh index.
    let Some(repo) = sample_repo() else {
        eprintln!("skipping: git CLI unavailable");
        return;
    };
    let gix = Repository::discover(repo.path()).expect("open fixture");

    let conn = open_indexed(std::path::Path::new(":memory:")).expect("in-memory index");
    let storage = SqliteStorageProvider::new(&conn);
    let indexer = Indexer::new(&gix, &storage);
    let mut progress = NoopProgress;
    indexer.scan(&mut progress).expect("scan");

    let (authors, tree): (i64, i64) = conn
        .borrow()
        .query_row(
            "SELECT count(DISTINCT author_id), count(*) FROM commits \
             WHERE author_id IS NOT NULL AND tree_oid IS NOT NULL AND tree_oid != ''",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("author/tree stats");
    assert_eq!(authors, 1, "fixture has one author; author_id must be set");
    assert_eq!(
        tree, 3,
        "every commit must carry its tree oid (got {tree}/3)"
    );

    // Author rows themselves must be real identities, not a NULL-adjacent
    // placeholder.
    let name: String = conn
        .borrow()
        .query_row("SELECT name FROM authors LIMIT 1", [], |row| row.get(0))
        .expect("author name");
    assert_eq!(name, "IT Tester");
}

#[test]
fn analysis_cache_writes_real_languages() {
    // The language column backing the Overview breakdown must hold real
    // extensions after the index + analysis cache are populated together.
    let Some(repo) = sample_repo() else {
        eprintln!("skipping: git CLI unavailable");
        return;
    };
    let gix = Repository::discover(repo.path()).expect("open fixture");

    let conn = open_indexed(std::path::Path::new(":memory:")).expect("in-memory index");
    let storage = SqliteStorageProvider::new(&conn);
    let indexer = Indexer::new(&gix, &storage);
    let mut progress = NoopProgress;
    indexer.scan(&mut progress).expect("scan");

    let analysis = gitx_analysis::analyze_repository(&gix).expect("analyze fixture");
    let mut conn = conn.borrow_mut();
    let head = gix.head_commit_id().expect("head");
    gitx_analysis::cache::store(&mut conn, &gix, &analysis, &head.to_string())
        .expect("store cache");

    let languages: Vec<(String, i64)> = conn
        .prepare("SELECT language, count(*) FROM files WHERE is_current = 1 GROUP BY language")
        .expect("prepare")
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .expect("query")
        .collect::<Result<_, _>>()
        .expect("collect");
    assert!(
        languages.iter().any(|(lang, _)| lang == "rs"),
        "sample_repo has src/lib.rs and main.rs; languages must include 'rs', got {languages:?}"
    );
    assert!(
        languages.iter().all(|(lang, _)| lang != "none"),
        "no file may carry the 'none' placeholder, got {languages:?}"
    );
}
