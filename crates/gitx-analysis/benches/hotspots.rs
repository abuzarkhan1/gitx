//! Micro-benchmarks for the analysis hot paths (docs/13 performance).
//!
//! Run with: `cargo bench -p gitx-analysis`

use criterion::{Criterion, criterion_group, criterion_main};
use gitx_analysis::classification::classify_commit_message;
use gitx_analysis::hotspots::calculate_hotspot_score;

fn bench_hotspot_score(c: &mut Criterion) {
    c.bench_function("calculate_hotspot_score", |b| {
        b.iter(|| calculate_hotspot_score(55.0, 40.0, 20.0, 85.0, 60.0, 30))
    });
}

fn bench_commit_classification(c: &mut Criterion) {
    c.bench_function("classify_commit_message", |b| {
        b.iter(|| classify_commit_message("fix: resolve workspace manager crash on branch switch"))
    });
}

criterion_group!(benches, bench_hotspot_score, bench_commit_classification);
criterion_main!(benches);
