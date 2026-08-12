//! Application-services layer (docs/04 §6) + FTS search (docs/11):
//! exercises `RepositoryService`, `IndexService`, `AnalysisService` and
//! `SearchService` against a real fixture repository.

#[path = "../common/mod.rs"]
mod common;

use common::FixtureRepo;
use gitx_services::{
    AnalysisService, IndexService, RepositoryService, SearchOptions, SearchService,
};

fn repo() -> Option<FixtureRepo> {
    let repo = FixtureRepo::new("services")?;
    repo.write(
        "src/lib.rs",
        "pub fn hello() { println!(\"hi workspace\"); }\n",
    );
    repo.commit("feat: workspace-aware hello");
    repo.write("README.md", "# demo\n");
    repo.commit("docs: workspace readme");
    repo.write("src/lib.rs", "pub fn hello() { println!(\"hi there\"); }\n");
    repo.commit("fix: workspace message");
    Some(repo)
}

#[test]
fn index_service_scans_and_reports_fresh_state() {
    let Some(repo) = repo() else {
        eprintln!("git unavailable; skipping");
        return;
    };
    let git = gitx_git::Repository::discover(repo.path()).unwrap();
    let index = IndexService::new(&git);
    assert_eq!(index.scan(false).unwrap(), 3, "three commits indexed");

    let state = RepositoryService::new(&git).state();
    assert_eq!(
        state.index,
        gitx_services::IndexState::Indexed,
        "fresh index after scan"
    );

    let status = index.status().unwrap();
    assert_eq!(status, 3);
}

#[test]
fn repository_service_reports_degraded_states() {
    let Some(repo) = repo() else {
        return;
    };
    let git = gitx_git::Repository::discover(repo.path()).unwrap();
    let service = RepositoryService::new(&git);
    // No index yet → Unsupported.
    assert_eq!(
        service.state().index,
        gitx_services::IndexState::Unsupported
    );

    let index = IndexService::new(&git);
    index.scan(false).unwrap();
    assert_eq!(service.state().index, gitx_services::IndexState::Indexed);

    // Corrupt the index → Failed.
    std::fs::write(index.index_path(), b"not a database").unwrap();
    assert_eq!(service.state().index, gitx_services::IndexState::Failed);
}

#[test]
fn analysis_service_serves_cached_results() {
    let Some(repo) = repo() else {
        return;
    };
    let git = gitx_git::Repository::discover(repo.path()).unwrap();
    let index = IndexService::new(&git);
    index.scan(false).unwrap();

    let analysis = AnalysisService::new(&git)
        .analyze(true, gitx_analysis::hotspots::HotspotWeights::default())
        .unwrap();
    assert_eq!(analysis.total_commits, 3);
    assert!(
        analysis
            .files
            .iter()
            .any(|f| f.path.ends_with("src/lib.rs")),
        "cached analysis includes src/lib.rs"
    );
}

#[test]
fn search_service_queries_across_scopes() {
    let Some(repo) = repo() else {
        return;
    };
    let git = gitx_git::Repository::discover(repo.path()).unwrap();
    let index = IndexService::new(&git);
    index.scan(false).unwrap();

    let service = SearchService::new(&git);
    // Commit-message hit.
    let hits = service
        .search(
            "\"workspace\"",
            &SearchOptions {
                commits: true,
                ..Default::default()
            },
        )
        .unwrap();
    assert!(
        hits.iter().any(|h| h.scope == "commit"),
        "commit hits for workspace, got {hits:?}"
    );

    // File-path hit.
    let hits = service
        .search(
            "\"README\"",
            &SearchOptions {
                files: true,
                ..Default::default()
            },
        )
        .unwrap();
    assert!(
        hits.iter().any(|h| h.scope == "file"),
        "file hit for README"
    );
}

#[test]
fn full_scan_equals_incremental_from_empty() {
    // Index-consistency (docs/14 §8): the incremental indexer walking from an
    // empty index must produce the same commit/ref set as the manual
    // `build_index` full build, and both must match git's own count.
    let Some(repo) = repo() else {
        return;
    };
    let git = gitx_git::Repository::discover(repo.path()).unwrap();

    // git's ground truth.
    let head = git.head_commit_id().unwrap();
    let git_count = git.rev_walk(head).unwrap().count();

    // Path A: incremental Indexer from empty (what `gitx scan` runs).
    let path = repo.path().join(".git/gitx/index.sqlite");
    let conn = gitx_storage::open_indexed(&path).unwrap();
    let storage = gitx_storage::SqliteStorageProvider::new(&conn);
    let indexer = gitx_index::Indexer::new(&git, &storage);
    indexer.scan(&mut gitx_index::NoopProgress).unwrap();
    let a: i64 = conn
        .borrow()
        .query_row("SELECT count(*) FROM commits", [], |row| row.get(0))
        .unwrap();
    assert_eq!(a as usize, git_count, "incremental-from-empty matches git");

    // Path B: manual full build (`gitx index rebuild` runs this).
    let path_b = repo.path().join(".git/gitx/index-b.sqlite");
    let mut conn_b = rusqlite::Connection::open(&path_b).unwrap();
    gitx_services::index::build_index(&mut conn_b, &git).unwrap();
    let b: i64 = conn_b
        .query_row("SELECT count(*) FROM commits", [], |row| row.get(0))
        .unwrap();
    assert_eq!(b as usize, git_count, "full build matches git");
    assert_eq!(a, b, "incremental-from-empty and full build agree");

    // Branch/ref agreement.
    let branches_a: i64 = conn
        .borrow()
        .query_row("SELECT count(*) FROM branches", [], |row| row.get(0))
        .unwrap();
    let branches_b: i64 = conn_b
        .query_row("SELECT count(*) FROM branches", [], |row| row.get(0))
        .unwrap();
    assert_eq!(branches_a, branches_b, "branch rows agree");
    let _ = std::fs::remove_file(&path_b);
}
