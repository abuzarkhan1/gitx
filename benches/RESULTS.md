# Benchmark results

Run with `scripts/bench.sh` (appends a timestamped section below). Criterion
flags: `--warm-up-time 1 --measurement-time 3`. CI keeps benches compile-only
(`cargo bench --workspace --no-run`) per docs/13 §10; this file is the
recorded baseline for drift comparison.

Fixtures are generated in temp dirs by `crates/gitx-services/benches/operations.rs`:
120-commit single-file baseline, 400-commit deep history, 400-commit
merge-heavy + a persistent diverged branch, and a 500-commit × 20-file medium
repo.

## 2026-08-13 — Mac

| Crate | Bench | Mean |
|---|---|---|
| gitx-analysis | calculate_hotspot_score | 1.8806 ns |
| gitx-analysis | classify_commit_message | 24.774 ns |
| analysis | analyze_repository_120_commits | 32.488 ms |
| analysis | analyze_regressions_120_commits | 31.755 ms |
| services | index_scan_120_commits | 42.309 ms |
| services | stats_from_index_120_commits | 236.02 µs |
| services | search_fts_120_commits | 352.42 µs |
| history | file_lineage_deep_history | 109.08 ms |
| branches | branch_intelligence_all_local | 24.417 ms |
| search | search_fts_medium_repo | 411.69 µs |

