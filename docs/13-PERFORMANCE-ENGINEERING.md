# Performance Engineering

## 1. Philosophy

GitX is an interactive developer tool. Performance is a product feature.

## 2. Main strategy

Use:

```text
incremental indexing
persistent cache
batched SQLite writes
parallel analysis
lazy TUI loading
bounded memory
```

## 3. Performance targets

Initial targets are engineering budgets, not guarantees:

### Startup

If a valid index exists:

```text
TUI should become usable in roughly sub-second to low-single-digit-second range on normal developer hardware.
```

### Cached queries

Common queries should normally feel instantaneous:

- repository overview
- branch list
- contributor list
- hotspot list
- recent timeline
- indexed search

### Incremental refresh

For a small number of new commits, refresh should process only affected history and derived metrics.

## 4. Large repository behavior

Never load the complete repository history into RAM if a streaming/query approach is sufficient.

Use:

- iterators
- pagination
- bounded caches
- database queries
- lazy rendering

## 5. SQLite

Use:

- prepared statements
- transactions
- WAL where appropriate
- indexes on frequent query fields
- FTS5 for text search

Benchmark WAL and journal settings before locking them into defaults.

## 6. Concurrency

Parallelize CPU-heavy independent operations.

Do not create uncontrolled threads per file/commit.

Use a bounded worker model.

## 7. TUI performance

Avoid recalculating analysis during rendering.

Preferred:

```text
background task
→ result/cache
→ UI reads immutable result
```

Not:

```text
render()
→ scan Git
→ calculate metrics
→ render
```

## 8. Memory

Large diffs should be paginated or streamed.

Do not keep every commit diff in memory.

## 9. Benchmarks

Benchmark repositories are generated deterministically in temp dirs by
`crates/gitx-services/benches/operations.rs` (the git CLI is used only to
*generate* fixtures; timed loops run through gix):

- tiny baseline: 120 commits / 1 file (`services/*_120_commits`)
- medium: 500 commits / 20 files (`search/search_fts_medium_repo`)
- long-history: 400 commits / 1 file (`history/file_lineage_deep_history`)
- merge-heavy: 400 commits with periodic two-parent merges
  (`branches/branch_intelligence_all_local`)
- rename-heavy: 400 commits with periodic file renames

Benchmarked operations: initial index scan, stats read, FTS search (small +
medium repos), hotspot/regression analysis (`crates/gitx-analysis/benches`),
file-history lineage, and branch analysis. TUI data preparation is covered by
`scripts/verify-tui.sh`'s lazy-loading startup checks (see §7). Run everything
with `scripts/bench.sh`; results append to `benches/RESULTS.md`.

## 10. Regression policy

A significant performance regression must be investigated before release.
`benches/RESULTS.md` holds the recorded baseline (host + date per run, see
`scripts/bench.sh`). Compare a new run's means against the stored baseline; a
>10% mean regression on the same host warrants investigation before release.
CI stays compile-only (`cargo bench --workspace --no-run`) so benches never
slow down the pipeline — they are a release-gate measurement, not a CI check.
