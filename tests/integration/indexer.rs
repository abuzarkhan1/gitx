//! Exercises the gitx-index Indexer end-to-end with real providers
//! (docs/09: full scan + incremental refresh).

use gitx_git::Repository;
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
