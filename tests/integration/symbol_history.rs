//! Exercises `gitx symbols history` end-to-end (docs/21 Stage 6): a symbol
//! must be reported Added at birth, Moved when its line shifts, and Removed
//! when it disappears.

use gitx_analysis::symbol_history::{symbol_history, SymbolAction};
use gitx_git::Repository;
use std::path::Path;

#[path = "../common/mod.rs"]
mod common;
use common::FixtureRepo;

#[test]
fn symbol_history_tracks_add_move_remove() {
    let Some(repo) = FixtureRepo::new("symbol-history") else {
        eprintln!("skipping: git CLI unavailable");
        return;
    };
    repo.write("src/lib.rs", "fn helper() {}\n");
    repo.commit("feat: add helper");
    // Move helper down by prepending a line.
    repo.write("src/lib.rs", "use std::fmt;\nfn helper() {}\n");
    repo.commit("refactor: import fmt");
    // Rename the function away.
    repo.write("src/lib.rs", "use std::fmt;\nfn helper2() {}\n");
    repo.commit("refactor: rename helper to helper2");

    let gix = Repository::discover(repo.path()).expect("open fixture");
    let events = symbol_history(&gix, "helper", None).expect("symbol history");

    let removed = events
        .iter()
        .find(|e| matches!(e.action, SymbolAction::Removed { .. }))
        .expect("helper removed at the rename commit");
    assert!(
        matches!(removed.action, SymbolAction::Removed { line: 2 }),
        "helper was on line 2 when removed, got {:?}",
        removed.action
    );
    assert_eq!(removed.file, Path::new("src/lib.rs"));

    let moved = events
        .iter()
        .find(|e| matches!(e.action, SymbolAction::Moved { .. }))
        .expect("helper moved when the import was prepended");
    assert!(
        matches!(
            moved.action,
            SymbolAction::Moved {
                from_line: 1,
                to_line: 2
            }
        ),
        "expected move 1 -> 2, got {:?}",
        moved.action
    );

    let added = events
        .iter()
        .find(|e| matches!(e.action, SymbolAction::Added { .. }))
        .expect("helper added at birth");
    assert!(
        matches!(added.action, SymbolAction::Added { line: 1 }),
        "expected added at line 1, got {:?}",
        added.action
    );

    // Chronology: added <= moved <= removed (fixture commits may share a
    // timestamp, so compare times rather than list order).
    assert!(added.time <= moved.time, "added before moved");
    assert!(moved.time <= removed.time, "moved before removed");
}

#[test]
fn symbol_history_reports_renamed_symbol_as_new_add() {
    let Some(repo) = FixtureRepo::new("symbol-history-rename") else {
        eprintln!("skipping: git CLI unavailable");
        return;
    };
    repo.write("src/lib.rs", "fn helper() {}\n");
    repo.commit("feat: add helper");
    repo.write("src/lib.rs", "fn helper2() {}\n");
    repo.commit("refactor: rename helper to helper2");

    let gix = Repository::discover(repo.path()).expect("open fixture");
    let events = symbol_history(&gix, "helper2", None).expect("symbol history");
    assert_eq!(
        events.len(),
        1,
        "helper2 only has its birth, got {events:?}"
    );
    assert!(
        matches!(events[0].action, SymbolAction::Added { line: 1 }),
        "helper2 added at line 1, got {:?}",
        events[0].action
    );
}

#[test]
fn symbol_history_empty_for_unknown_symbol() {
    let Some(repo) = FixtureRepo::new("symbol-history-empty") else {
        eprintln!("skipping: git CLI unavailable");
        return;
    };
    repo.write("src/lib.rs", "fn helper() {}\n");
    repo.commit("feat: add helper");

    let gix = Repository::discover(repo.path()).expect("open fixture");
    let events = symbol_history(&gix, "no_such_fn", None).expect("symbol history");
    assert!(events.is_empty());
}

#[test]
fn symbol_history_respects_path_scope() {
    let Some(repo) = FixtureRepo::new("symbol-history-path") else {
        eprintln!("skipping: git CLI unavailable");
        return;
    };
    repo.write("src/lib.rs", "fn helper() {}\n");
    repo.commit("feat: add helper");
    repo.write("other.txt", "fn helper() {}\n");
    repo.commit("chore: unrelated file");

    let gix = Repository::discover(repo.path()).expect("open fixture");
    // Path scope excludes the txt file (no extractor anyway); scoping to a
    // non-existent prefix must yield no events.
    let events = symbol_history(&gix, "helper", Some(Path::new("nope/"))).expect("symbol history");
    assert!(events.is_empty());
}
