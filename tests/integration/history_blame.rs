use gitx_git::Repository;
use gitx_history::timeline::{HistoryService, TimelineOptions};
use std::path::PathBuf;

#[path = "../common/mod.rs"]
mod common;
use common::{sample_repo, FixtureRepo};

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

#[test]
fn timeline_filters_by_committer_and_merges() {
    let Some(repo) = FixtureRepo::new("timeline-filters") else {
        eprintln!("skipping: git CLI unavailable");
        return;
    };
    repo.write("src/lib.rs", "pub fn a() {}\n");
    repo.commit("feat: base");
    repo.git(&["checkout", "-q", "-b", "feature"]);
    repo.git(&["config", "user.email", "bot@example.com"]);
    repo.git(&["config", "user.name", "Deploy Bot"]);
    repo.write("src/lib.rs", "pub fn a() {}\npub fn b() {}\n");
    repo.commit("feat: feature work");
    repo.git(&["checkout", "-q", "main"]);
    repo.git(&["merge", "-q", "--no-ff", "feature", "-m", "merge: feature"]);

    let gix = Repository::discover(repo.path()).expect("open fixture");
    let service = HistoryService::new(&gix);

    // Committer filter matches the feature + merge commits.
    let by_committer = service
        .timeline(TimelineOptions {
            committer: Some("bot@example.com".into()),
            ..Default::default()
        })
        .expect("timeline");
    assert_eq!(
        by_committer.len(),
        2,
        "expected feature + merge, got {by_committer:?}"
    );
    assert!(
        by_committer
            .iter()
            .all(|c| c.committer.email == "bot@example.com"),
        "committer filter leaked: {by_committer:?}"
    );

    // Merge-only filter returns exactly the merge commit.
    let merges = service
        .timeline(TimelineOptions {
            merges_only: true,
            ..Default::default()
        })
        .expect("timeline");
    assert_eq!(merges.len(), 1);
    assert_eq!(merges[0].parents.len(), 2);
    assert!(merges[0].message.contains("merge: feature"));

    // No-merges excludes it.
    let no_merges = service
        .timeline(TimelineOptions {
            no_merges: true,
            ..Default::default()
        })
        .expect("timeline");
    assert!(
        no_merges.iter().all(|c| c.parents.len() <= 1),
        "merge commit leaked through no_merges: {no_merges:?}"
    );
}
