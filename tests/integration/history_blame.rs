use gitx_git::Repository;
use gitx_history::timeline::{HistoryService, TimelineOptions};
use std::path::PathBuf;

#[path = "../common/mod.rs"]
mod common;
use common::sample_repo;

#[test]
fn timeline_returns_commits_newest_first() {
    let Some(repo) = sample_repo() else {
        eprintln!("skipping: git CLI unavailable");
        return;
    };
    let gix = Repository::discover(repo.path()).expect("open fixture");
    let service = HistoryService::new(&gix);
    let commits = service
        .timeline(TimelineOptions::default())
        .expect("timeline");

    assert_eq!(commits.len(), 3);
    assert!(commits[0].message.contains("docs: add readme"));
    assert!(commits[2].message.contains("feat: initial scaffold"));
}

#[test]
fn timeline_path_filter_only_returns_touching_commits() {
    let Some(repo) = sample_repo() else {
        eprintln!("skipping: git CLI unavailable");
        return;
    };
    let gix = Repository::discover(repo.path()).expect("open fixture");
    let service = HistoryService::new(&gix);

    let commits = service
        .timeline(TimelineOptions {
            path: Some(PathBuf::from("src/lib.rs")),
            ..Default::default()
        })
        .expect("timeline");
    assert_eq!(commits.len(), 2, "README commit must not touch src/lib.rs");
    assert!(
        commits
            .iter()
            .all(|c| !c.message.contains("docs: add readme")),
        "README commit leaked through the path filter"
    );
}

#[test]
fn blame_attributes_lines_to_introducing_commits() {
    let Some(repo) = sample_repo() else {
        eprintln!("skipping: git CLI unavailable");
        return;
    };
    let gix = Repository::discover(repo.path()).expect("open fixture");
    let service = HistoryService::new(&gix);

    let result = service
        .blame(PathBuf::from("src/lib.rs"), None)
        .expect("blame");
    assert_eq!(result.lines.len(), 2);

    // Line 1 was rewritten by "fix: hello prints", line 2 added there too.
    let first = gix.find_commit(result.lines[0].commit_id).expect("commit");
    assert!(first.message.contains("fix: hello prints"));
    let second = gix.find_commit(result.lines[1].commit_id).expect("commit");
    assert!(second.message.contains("fix: hello prints"));
}

#[test]
fn diff_stats_are_real() {
    let Some(repo) = sample_repo() else {
        eprintln!("skipping: git CLI unavailable");
        return;
    };
    let gix = Repository::discover(repo.path()).expect("open fixture");

    let head = gix.head_commit_id().expect("head");
    let head_commit = gix.find_commit(head).expect("head commit");
    let parent = gix
        .find_commit(head_commit.parents[0])
        .expect("parent commit");

    let changes = gix
        .diff_tree_to_tree(Some(parent.tree_id), head_commit.tree_id)
        .expect("diff");
    // The docs commit adds README.md with one line.
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].insertions, 1);
    assert_eq!(changes[0].deletions, 0);
}
