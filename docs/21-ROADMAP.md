# Roadmap

## Stage 1 — Foundation

Goal:

> Read Git correctly.

Deliver:

- Rust workspace
- gix integration
- repository discovery
- commits
- refs
- branches
- tags
- diffs
- authors

## Stage 2 — Index

Goal:

> Make repository data persistent and queryable.

Deliver:

- SQLite
- migrations
- initial scan
- incremental refresh
- FTS
- index validation

## Stage 3 — Explorer

Goal:

> Make history easy to understand.

Deliver:

- TUI
- timeline
- commit explorer
- file archaeology
- branch explorer
- search

## Stage 4 — Intelligence

Goal:

> Explain where repository maintenance is concentrated.

Deliver:

- churn
- hotspots
- ownership
- risk
- repository health score
- contributor analysis
- branch intelligence

## Stage 5 — Archaeology and recovery

Goal:

> Understand old and lost repository state.

Deliver:

- reflog
- unreachable objects
- deleted branch history
- release comparison
- historical architecture
- line-level history (blame)

## Stage 6 — Structural intelligence

Goal:

> Understand how the codebase itself evolved.

Deliver:

- dependency graph
- architecture graph
- language analyzers
- symbol history
- structural change detection

## Stage 7 — Scale and polish

Goal:

> Make GitX excellent on real repositories.

Deliver:

- performance optimization
- large repository support
- memory tuning
- polished TUI
- cross-platform releases
- stable JSON contracts
- comprehensive documentation

## Feature freeze principle

Do not add unrelated features merely because they are technically interesting.

GitX wins through depth in repository intelligence, not breadth.
