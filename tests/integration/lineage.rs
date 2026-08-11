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
    assert_eq!(
        lineage.history[1].action,
        gitx_history::FileAction::Added,
        "second node should be the original add"
    );
}
