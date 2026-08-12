//! Benchmarks for the repository-analysis pipeline (docs/13 §9): full
//! analysis and regression analysis over a generated 100-commit fixture.
//!
//! Run: `cargo bench -p gitx-analysis`.

use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use std::path::PathBuf;
use std::process::Command;

/// Build a deterministic 120-commit fixture repository in a temp dir and
/// return its path. The git CLI is used only to generate the fixture.
fn fixture() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "gitx-bench-{}-{}",
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
        let file = if i % 3 == 0 { "src/a.rs" } else { "src/b.rs" };
        let content = format!("// change {i}\npub fn f{i}() {{}}\n");
        std::fs::write(dir.join(file), content).unwrap();
        run(&["add", "-A"]);
        let msg = if i % 5 == 0 {
            format!("fix: change {i}")
        } else {
            format!("feat: change {i}")
        };
        run(&["commit", "-q", "-m", &msg]);
    }
    dir
}

fn bench_analysis(c: &mut Criterion) {
    let dir = fixture();
    let mut group = c.benchmark_group("analysis");
    group.sample_size(10);

    group.bench_function("analyze_repository_120_commits", |b| {
        b.iter_batched(
            || gitx_git::Repository::discover(&dir).unwrap(),
            |repo| gitx_analysis::analyze_repository(&repo).unwrap(),
            BatchSize::SmallInput,
        )
    });

    group.bench_function("analyze_regressions_120_commits", |b| {
        b.iter_batched(
            || gitx_git::Repository::discover(&dir).unwrap(),
            |repo| gitx_analysis::analyze_regressions(&repo, Some(120)).unwrap(),
            BatchSize::SmallInput,
        )
    });

    group.finish();
    let _ = std::fs::remove_dir_all(&dir);
}

criterion_group!(benches, bench_analysis);
criterion_main!(benches);
