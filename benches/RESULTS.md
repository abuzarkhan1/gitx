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

## 2026-08-13 — Large-repo validation (real clone)

Dogfooding pass against a real large repository: the `clap` source clone at
`/tmp/clap-dogfood` (github.com/clap-rs/clap), **9,070 commits**, ~24 MB `.git`,
release build, warm OS cache, on the 2026-era MacBook. Measured with
`/usr/bin/time -p`; TUI paint via tmux poll.

Three fixes landed during this pass, so numbers are recorded before/after:

1. **gix decoded-object cache** (`gitx_git::Repository`, docs/13 §4): gix leaves
   its cache unset, so walks re-decompressed pack objects. A 128 MB memory-
   capped cache turned repeated reads into hash lookups.
2. **Boundary-stop incremental walk** (`GitProvider::walk_commits` + engine,
   docs/13 §3): the provider never descends into commits already in the index,
   so refresh touches O(new commits) objects instead of the whole history.
3. **Analysis-cache freshness skip** (`IndexService::scan_with`): a no-op
   refresh no longer re-runs the full live analysis when the cache is already
   fresh for HEAD.

| Scenario | Before | After |
|---|---|---|
| first-run `gitx health` (auto-refresh: full scan + analysis) | ~60 s | **3.66 s** |
| no-op `gitx refresh` | ~16.5 s | **0.03 s** |
| `gitx refresh` after +1 commit | ~16.5 s | **2.9 s** → **0.04 s** |
| cached `gitx health` | — | **0.00 s** |
| cached `gitx hotspots` | — | **0.01 s** |
| `gitx search` (lazy in-memory index) | — | **0.26 s** |
| TUI Overview first paint (index-backed) | — | **~0.95 s** |

Cached reads stay sub-second on the 9k-commit repo (docs/13 §3 targets), and
TUI startup is sub-second with a valid index.

## 2026-08-13 — Incremental analysis cache (same clap clone)

Fourth fix: the analysis cache was keyed to HEAD and recomputed from scratch
on every move — the `+1 commit` case above still paid an O(history) analysis
walk (~3 s). A new incremental path (`gitx_analysis::incremental`, schema v4)
applies the new commits' delta to the persisted per-file aggregates, then
re-normalizes hotspot/risk/health across the current file set. File-level
metrics stay bit-exact (verified by an integration test comparing against a
fresh full analysis); windowed scoring signals read the persisted columns,
matching the cache path's existing fidelity, and a full `gitx index rebuild`
reconciles windowed drift. Falls back to the full pipeline when preconditions
fail (no cache, rewritten history, >200 new commits, analysis head unreachable).

| Scenario | Before | After |
|---|---|---|
| `gitx refresh` after +1 commit (real content delta) | 2.9 s | **0.04 s** |
| `gitx refresh` after +1 empty commit | 2.9 s | **0.07 s** |
| cached `gitx health` after the refresh | 0.00 s | **0.00 s** |

Evidence counters were spot-checked live across successive increments
(commits 9070 → 9071, total_changes 46261 → 46262, recent_changes 0 → 1,
health 69.3 → 72.3) and `analysis_head` tracks HEAD.


Index correctness on the large repo was spot-checked live: 9,070 commits / 650
contributors / 631 files / 634 tags, and the TUI Overview rendered all four
from the index rather than recomputing.

