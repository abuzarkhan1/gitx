//! Benchmarks for the services layer (docs/13 §9): initial index scan,
//! FTS search, and index-backed statistics over a generated fixture.
//!
//! Run: `cargo bench -p gitx-services`.

use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use gitx_services::{IndexService, RepositoryService, SearchOptions, SearchService};
use std::path::PathBuf;
use std::process::Command;

/// Build a deterministic 120-commit fixture repository in a temp dir and
/// return its path. The git CLI is used only to generate the fixture.
fn fixture() -> PathBuf {
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
    for i in 0..120 {
        std::fs::write(
            dir.join("src/app.rs"),
            format!("// change {i}\npub fn f{i}() {{}}\n"),
        )
        .unwrap();
        run(&["add", "-A"]);
        run(&["commit", "-q", "-m", &format!("feat: workspace change {i}")]);
    }
    dir
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

criterion_group!(benches, bench_services);
criterion_main!(benches);
