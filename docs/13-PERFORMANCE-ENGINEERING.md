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

Maintain benchmark repositories:

- tiny
- medium
- large
- merge-heavy
- rename-heavy
- long-history
- binary-heavy

Benchmark:

- initial index
- refresh
- search
- hotspot calculation
- branch analysis
- file history
- TUI data preparation

## 10. Regression policy

A significant performance regression must be investigated before release.

Store benchmark results where possible.
