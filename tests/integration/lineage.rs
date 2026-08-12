//! Exercises file lineage rename-following end-to-end (docs/10 file archaeology).

use gitx_git::Repository;
use gitx_history::timeline::HistoryService;

#[path = "../common/mod.rs"]
mod common;
use common::FixtureRepo;

#[test]
fn lineage_follows_rename_backward() {
    let Some(repo) = FixtureRepo::new("lineage") else {
        eprintln!("skipping: git CLI unavailable");
        return;
    };
    repo.write("src/lib.rs", "pub fn a() {}\n");
    repo.commit("feat: add lib");

    // Rename lib.rs -> core.rs (a real rename, no content change).
    repo.git(&["mv", "src/lib.rs", "src/core.rs"]);
    repo.commit("refactor: rename lib to core");

    let gix = Repository::discover(repo.path()).expect("open fixture");
    let service = HistoryService::new(&gix);
    let lineage = service
        .get_file_lineage(std::path::PathBuf::from("src/core.rs"), None)
        .expect("lineage");

    // Newest first: rename, then add.
    assert_eq!(
        lineage.history.len(),
        2,
        "expected rename + add, got {lineage:?}"
    );
    assert!(
        matches!(
            &lineage.history[0].action,
            gitx_history::FileAction::Renamed { from }
                if from == std::path::Path::new("src/lib.rs")
        ),
        "first node should be the rename, got {:?}",
        lineage.history[0].action
    );
    // The original add may now carry copy-of detection (blob-equality,
    // docs/02 §2 copies): only the action kind matters here.
    assert!(
        matches!(
            lineage.history[1].action,
            gitx_history::FileAction::Added { .. }
        ),
        "second node should be the original add"
    );
}

#[test]
fn copied_change_reports_copy_source() {
    let Some(repo) = FixtureRepo::new("lineage-copy") else {
        eprintln!("skipping: git CLI unavailable");
        return;
    };
    repo.write("src/util.rs", "pub fn helper() {}\n");
    repo.commit("feat: util");
    repo.write("src/copy.rs", "pub fn helper() {}\n");
    repo.commit("feat: copy util");

    let gix = Repository::discover(repo.path()).expect("open fixture");
    let lineage = HistoryService::new(&gix)
        .get_file_lineage(std::path::PathBuf::from("src/copy.rs"), None)
        .expect("lineage");
    let added = lineage
        .history
        .iter()
        .find(|n| matches!(n.action, gitx_history::FileAction::Added { .. }))
        .unwrap_or_else(|| panic!("copy.rs has an Added node, got {lineage:?}"));
    match &added.action {
        gitx_history::FileAction::Added { copy_of } => {
            assert_eq!(
                copy_of.as_deref(),
                Some(std::path::Path::new("src/util.rs"))
            )
        }
        other => panic!("expected Added with copy source, got {other:?}"),
    }
}

#[test]
fn merge_touch_is_marked_via_merge() {
    let Some(repo) = FixtureRepo::new("lineage-merge") else {
        eprintln!("skipping: git CLI unavailable");
        return;
    };
    repo.write("shared.txt", "v1\n");
    repo.commit("feat: shared");
    repo.git(&["checkout", "-q", "-b", "feature"]);
    repo.write("shared.txt", "v1\nfeature\n");
    repo.commit("feat: feature change");
    repo.git(&["checkout", "-q", "main"]);
    repo.git(&["merge", "-q", "--no-ff", "feature", "-m", "merge: feature"]);

    let gix = Repository::discover(repo.path()).expect("open fixture");
    let lineage = HistoryService::new(&gix)
        .get_file_lineage(std::path::PathBuf::from("shared.txt"), None)
        .expect("lineage");
    assert!(
        lineage.history.iter().any(|n| n.via_merge),
        "the merge commit touching the file must be marked, got {lineage:?}"
    );
}
