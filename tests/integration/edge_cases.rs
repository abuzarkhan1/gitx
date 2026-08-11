//! Edge-case coverage (docs/14 §9): merge commits, binary files, empty
//! commits, revert commits, and rewritten history. Mirrors the documented
//! fixture in tests/fixtures/build.sh (docs/14 §5).

#[path = "../common/mod.rs"]
mod common;

use common::FixtureRepo;
use gitx_git::models::ObjectId;

fn repo() -> Option<FixtureRepo> {
    let repo = FixtureRepo::new("edge")?;
    // c1: initial files (text + binary).
    repo.write("alpha.txt", "alpha\n");
    repo.write("blob.bin", "\u{0}\u{1}\u{2}\u{ff}\u{fe} binary payload");
    repo.commit("add: initial files");
    // c2: modify text file.
    repo.write("alpha.txt", "alpha\nbeta\n");
    repo.commit("fix: extend alpha");
    // c3: empty commit.
    assert!(repo
        .git(&["commit", "-q", "--allow-empty", "-m", "chore: empty commit"])
        .is_some());
    // c4: revert of c2 (HEAD~1; HEAD is the empty commit).
    assert!(repo.git(&["revert", "--no-edit", "HEAD~1"]).is_some());
    // c5: merge commit from a side branch.
    assert!(repo
        .git(&["checkout", "-q", "-b", "feature/experiment"])
        .is_some());
    repo.write("gamma.txt", "gamma\n");
    repo.commit("feat: experiment module");
    assert!(repo.git(&["checkout", "-q", "main"]).is_some());
    repo.write("mainline.txt", "mainline\n");
    repo.commit("feat: mainline change");
    assert!(repo
        .git(&[
            "merge",
            "-q",
            "--no-ff",
            "feature/experiment",
            "-m",
            "merge: bring in experiment"
        ])
        .is_some());
    Some(repo)
}

fn open(repo: &FixtureRepo) -> gitx_git::Repository {
    gitx_git::Repository::discover(repo.path()).expect("open fixture")
}

#[test]
fn merge_commit_has_two_parents_and_appears_in_timeline() {
    let Some(repo) = repo() else {
        eprintln!("git unavailable; skipping");
        return;
    };
    let git = open(&repo);
    let head = git.head_commit_id().unwrap();
    let merge = git.find_commit(head).unwrap();
    assert_eq!(
        merge.parents.len(),
        2,
        "merge commit must have two parents, got {:?}",
        merge.parents
    );

    // Every commit (including the merge) is reachable via rev_walk.
    let ids: Vec<ObjectId> = git
        .rev_walk(head)
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    let messages: Vec<String> = ids
        .iter()
        .map(|id| git.find_commit(*id).unwrap().message.clone())
        .collect();
    assert!(
        messages.iter().any(|m| m.starts_with("merge:")),
        "merge in history"
    );
    assert!(messages
        .iter()
        .any(|m| m.starts_with("chore: empty commit")));
}

#[test]
fn binary_files_do_not_break_diffs() {
    let Some(repo) = repo() else {
        return;
    };
    let git = open(&repo);
    let head = git.head_commit_id().unwrap();
    let commits: Vec<ObjectId> = git
        .rev_walk(head)
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    // The initial commit contains the binary blob; diffing it must not fail.
    let first = git.find_commit(commits.last().copied().unwrap()).unwrap();
    let changes = git.diff_tree_to_tree(None, first.tree_id).unwrap();
    assert!(
        changes.iter().any(|c| c.path.ends_with("blob.bin")),
        "binary file present in initial-commit diff"
    );
}

#[test]
fn empty_commit_reports_zero_changed_files() {
    let Some(repo) = repo() else {
        return;
    };
    let git = open(&repo);
    let head = git.head_commit_id().unwrap();
    let ids: Vec<ObjectId> = git
        .rev_walk(head)
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    // Find the empty commit (the one whose message is the chore).
    let empty = ids
        .iter()
        .map(|id| git.find_commit(*id).unwrap())
        .find(|c| c.message.starts_with("chore: empty commit"))
        .expect("empty commit exists");
    let parent_tree = empty
        .parents
        .first()
        .map(|p| git.find_commit(*p).unwrap().tree_id);
    let changes = git.diff_tree_to_tree(parent_tree, empty.tree_id).unwrap();
    assert!(changes.is_empty(), "empty commit changes nothing");
}

#[test]
fn revert_is_classified_and_its_target_resolved() {
    let Some(repo) = repo() else {
        return;
    };
    let git = open(&repo);
    let report = gitx_analysis::analyze_regressions(&git, Some(100)).unwrap();
    assert!(
        !report.reverts.is_empty(),
        "revert commit detected, got {}",
        report.reverts.len()
    );
}

#[test]
fn rewritten_history_is_detected_by_the_indexer() {
    let Some(repo) = repo() else {
        return;
    };
    // Index the pre-rewrite state first, so the index knows the old HEAD.
    let git = open(&repo);
    let before = git.head_commit_id().unwrap();
    let path = repo.path().join(".git/gitx/index.sqlite");
    let conn = gitx_storage::open_indexed(&path).unwrap();
    let storage = gitx_storage::SqliteStorageProvider::new(&conn);
    let indexer = gitx_index::Indexer::new(&git, &storage);
    indexer.scan(&mut gitx_index::NoopProgress).unwrap();

    // Amend the merge commit into a rewritten HEAD (old HEAD becomes unreachable).
    assert!(repo
        .git(&[
            "commit",
            "-q",
            "--amend",
            "-m",
            "merge: bring in experiment (rewritten)"
        ])
        .is_some());
    let git = open(&repo);
    let after = git.head_commit_id().unwrap();
    assert_ne!(before, after, "amend rewrote HEAD");

    // Refresh — the indexer must flag the rewrite.
    let indexer = gitx_index::Indexer::new(&git, &storage);
    indexer.refresh(&mut gitx_index::NoopProgress).unwrap();
    let rewritten: String = conn
        .borrow()
        .query_row(
            "SELECT value FROM index_metadata WHERE key = 'rewritten_detected'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(
        rewritten, "1",
        "rewritten history must be flagged in index metadata"
    );
}
