//! Benchmarks for the services layer (docs/13 §9): initial index scan,
//! FTS search, index-backed statistics, file-history lineage, and branch
//! analysis — over generated fixtures of increasing size and shape
//! (medium, merge-heavy, rename-heavy).
//!
//! Run: `cargo bench -p gitx-services` (or `scripts/bench.sh`).
//!
//! NOTE: the git CLI is used only to *generate* the fixtures; the timed
//! loops run entirely through gix.

use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use gitx_services::{IndexService, RepositoryService, SearchOptions, SearchService};
use std::path::PathBuf;
use std::process::Command;

/// Build a deterministic fixture repository in a temp dir and return its
/// path. `files` files, `commits` commits, touching a rotating file each
/// commit. `merge_every` inserts a two-parent `--no-ff` merge every N
/// commits; `rename_every` renames a file every N commits.
fn build_fixture(
    files: usize,
    commits: usize,
    merge_every: Option<usize>,
    rename_every: Option<usize>,
) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "gitx-svc-bench-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    let run = |args: &[&str]| {
        Command::new("git")
            .current_dir(&dir)
            .args(args)
            .output()
            .expect("git runs");
    };
    run(&["init", "-q", "-b", "main"]);
    run(&["config", "user.email", "bench@example.com"]);
    run(&["config", "user.name", "Bench"]);
    run(&["config", "commit.gpgsign", "false"]);
    for i in 0..commits {
        let file = format!("src/mod{}.rs", i % files);
        std::fs::write(
            dir.join(&file),
            format!("// change {i}\npub fn f{i}() {{}}\n"),
        )
        .unwrap();
        if let Some(every) = rename_every
            && i > 0
            && i % every == 0
        {
            // Rename the file we just touched (filesystem rename + add is
            // detected by the index as a rename).
            let old = format!("src/mod{}.rs", i % files);
            let new = format!("src/mod{}_renamed.rs", i % files);
            let _ = std::fs::rename(dir.join(old), dir.join(new));
            run(&["add", "-A"]);
            run(&["commit", "-q", "-m", &format!("refactor: rename {i}")]);
            continue;
        }
        if let Some(every) = merge_every
            && i > 0
            && i % every == 0
        {
            // Side branch with one commit, merged back with --no-ff so the
            // merge commit has two parents (merge-heavy history shape).
            run(&["checkout", "-q", "-b", &format!("side{i}")]);
            std::fs::write(
                dir.join("src/side.rs"),
                format!("// side {i}\npub fn side{i}() {{}}\n"),
            )
            .unwrap();
            run(&["add", "-A"]);
            run(&["commit", "-q", "-m", &format!("feat: side work {i}")]);
            run(&["checkout", "-q", "main"]);
            run(&[
                "merge",
                "-q",
                "--no-ff",
                "-m",
                &format!("merge: side {i}"),
                &format!("side{i}"),
            ]);
            run(&["branch", "-q", "-D", &format!("side{i}")]);
            continue;
        }
        run(&["add", "-A"]);
        run(&["commit", "-q", "-m", &format!("feat: workspace change {i}")]);
    }
    dir
}

/// The original small fixture: 120 commits over a single file (docs/13 §9
/// baseline). Kept so the established numbers stay comparable.
fn fixture() -> PathBuf {
    build_fixture(1, 120, None, None)
}

fn bench_services(c: &mut Criterion) {
    let dir = fixture();
    let mut group = c.benchmark_group("services");
    group.sample_size(10);

    group.bench_function("index_scan_120_commits", |b| {
        b.iter_batched(
            || gitx_git::Repository::discover(&dir).unwrap(),
            |repo| {
                // Fresh temp index path per iteration via a distinct cache dir
                // is not supported by the service; measure the incremental
                // refresh over the existing index instead.
                let svc = IndexService::new(&repo);
                svc.scan(true).unwrap()
            },
            BatchSize::SmallInput,
        )
    });

    group.bench_function("stats_from_index_120_commits", |b| {
        // Index once, then measure the read path.
        let repo = gitx_git::Repository::discover(&dir).unwrap();
        IndexService::new(&repo).scan(false).unwrap();
        b.iter_batched(
            || gitx_git::Repository::discover(&dir).unwrap(),
            |repo| RepositoryService::new(&repo).stats_from_index().unwrap(),
            BatchSize::SmallInput,
        )
    });

    group.bench_function("search_fts_120_commits", |b| {
        let repo = gitx_git::Repository::discover(&dir).unwrap();
        IndexService::new(&repo).scan(false).unwrap();
        b.iter_batched(
            || gitx_git::Repository::discover(&dir).unwrap(),
            |repo| {
                SearchService::new(&repo)
                    .search(
                        "\"workspace\"",
                        "workspace",
                        &SearchOptions {
                            commits: true,
                            files: true,
                            authors: true,
                            branches: true,
                            tags: true,
                            ..Default::default()
                        },
                    )
                    .unwrap()
            },
            BatchSize::SmallInput,
        )
    });

    group.finish();
    let _ = std::fs::remove_dir_all(&dir);
}

/// File-history lineage over a deep single-file history (docs/13 §9): the
/// walk follows every commit from HEAD back to the file's birth.
fn bench_file_lineage(c: &mut Criterion) {
    let dir = build_fixture(1, 400, None, None);
    let mut group = c.benchmark_group("history");
    group.sample_size(10);
    group.bench_function("file_lineage_deep_history", |b| {
        b.iter_batched(
            || gitx_git::Repository::discover(&dir).unwrap(),
            |repo| {
                let svc = gitx_history::timeline::HistoryService::new(&repo);
                svc.get_file_lineage("src/mod0.rs".into(), None).unwrap()
            },
            BatchSize::SmallInput,
        )
    });
    group.finish();
    let _ = std::fs::remove_dir_all(&dir);
}

/// Branch ahead/behind intelligence over every local branch of a
/// merge-heavy history (docs/13 §9): per-branch reachability diffs.
fn bench_branch_analysis(c: &mut Criterion) {
    let dir = build_fixture(10, 400, Some(25), None);
    // Keep a genuinely diverged branch alive (the merge fixture deletes its
    // side branches), so the analysis compares real reachability sets.
    let run = |args: &[&str]| {
        Command::new("git")
            .current_dir(&dir)
            .args(args)
            .output()
            .expect("git runs");
    };
    run(&["branch", "feature/diverged", "HEAD~20"]);
    let mut group = c.benchmark_group("branches");
    group.sample_size(10);
    group.bench_function("branch_intelligence_all_local", |b| {
        b.iter_batched(
            || gitx_git::Repository::discover(&dir).unwrap(),
            |repo| {
                let branches = repo.branches().unwrap();
                let current = repo.head_commit_id().ok().and_then(|id| {
                    branches
                        .iter()
                        .find(|b| !b.is_remote && b.target == id)
                        .cloned()
                });
                for branch in &branches {
                    if !branch.is_remote {
                        let _ = gitx_analysis::branch::branch_intelligence(
                            &repo,
                            branch,
                            current.as_ref(),
                        )
                        .unwrap();
                    }
                }
            },
            BatchSize::SmallInput,
        )
    });
    group.finish();
    let _ = std::fs::remove_dir_all(&dir);
}

/// FTS search over a medium repository (500 commits × 20 files): index
/// build is the setup, the timed loop is the query (docs/13 §9).
fn bench_fts_search_medium(c: &mut Criterion) {
    let dir = build_fixture(20, 500, None, None);
    let repo = gitx_git::Repository::discover(&dir).unwrap();
    IndexService::new(&repo).scan(false).unwrap();
    let mut group = c.benchmark_group("search");
    group.sample_size(10);
    group.bench_function("search_fts_medium_repo", |b| {
        b.iter_batched(
            || gitx_git::Repository::discover(&dir).unwrap(),
            |repo| {
                SearchService::new(&repo)
                    .search(
                        "\"change\"",
                        "change",
                        &SearchOptions {
                            commits: true,
                            files: true,
                            authors: true,
                            branches: true,
                            tags: true,
                            ..Default::default()
                        },
                    )
                    .unwrap()
            },
            BatchSize::SmallInput,
        )
    });
    group.finish();
    let _ = std::fs::remove_dir_all(&dir);
}

criterion_group!(
    benches,
    bench_services,
    bench_file_lineage,
    bench_branch_analysis,
    bench_fts_search_medium
);
criterion_main!(benches);
