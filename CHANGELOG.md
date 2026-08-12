# Changelog

All notable changes to GitX are documented here. Format follows
[Keep a Changelog](https://keepachangelog.com/); versions follow
[Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added
- Search: `--renames`/`--history` filters query rename history; new
  `--symbols`/`--directories` scopes (CLI + TUI); deterministic tiered
  ranking (exact → path/name → recent → FTS score); bounded code-content
  search with a HEAD-tree fallback for bare repositories and a cap note.
- Symbols: dependency-free heuristic extractor across 10+ languages with a
  new `gitx symbols` command; symbols are indexed by `scan`/`refresh` and
  searchable.
- Architecture milestones: `gitx architecture milestones` (initial commit,
  first tag, module additions, structural refactors, dependency-direction
  flips); `architecture diff` reports direction changes.
- Dependencies: direct/transitive classification, Cargo `[features]` and
  pnpm `catalogs:` support; new `gitx dependencies features` subcommand.
- Commit detail: classification, related history (shared-file overlap) and
  affected contributors in `gitx commit` and the TUI detail view.
- `gitx history --follow` follows renames via the lineage engine (was a
  no-op flag); copy lineage detected via blob equality at file birth.
- TUI: responsive sidebar (width-adaptive with truncated labels); loader
  progress with stage + step/total and Esc-to-cancel; structured error
  overlay with "Reason / Suggested action"; Recovery panel lists dangling
  trees/blobs; branch rows show divergence, shared files and staleness;
  file detail shows contributors, churn and hotspot metrics.
- Repository intelligence pipeline: hotspots, risk, ownership concentration,
  and the six-sub-score health composite (docs/10).
- Incremental indexer with SQLite persistence (`gitx scan` / `gitx refresh`).
- FTS5-backed search over commits, files, authors, branches and tags
  (`gitx search`).
- Recovery tooling: reflog inspection and unreachable-commit detection
  (`gitx recovery`, `gitx unreachable`).
- Dependency overview, history and diff from declared manifests
  (`gitx dependencies`, `gitx dependencies history|diff`).
- Release diffs between any two refs (`gitx release diff`).
- Interactive TUI (`gitx-tui`) with 14 panels: Overview, Timeline, Commits,
  Branches, Files, Contributors, Hotspots, Ownership, Architecture,
  Dependencies, Risk, Health, Recovery and live Search.
- Shell completions (`gitx completions <shell>`).
- TOML configuration (`gitx config show|init`).
- Language-aware complexity: hotspot/risk scoring uses the symbol
  extractor's function count (`symbols+loc`), with a labeled LOC fallback
  for binary/unknown files.
- `gitx symbols history <name>`: where and when a symbol was born, moved,
  renamed, or deleted across commit lineage.
- `gitx graph` call edges (heuristic `name(` scans) and a TUI Graph view
  (`g` key) with per-directory file/import/call counts.
- Advanced filters: `gitx timeline --committer/--merges/--no-merges` and
  `gitx search --until/--path` (CLI + TUI).
- Streamed `gitx diff A B` with `--stat` and `--path` modes, paged output
  for wide diffs.
- CSV export: `--csv` on the tabular commands (hotspots, risk, health,
  branches, contributors, ownership, timeline) via a shared deterministic
  writer.
- TUI lazy loading: the Overview paints Phase A data immediately while
  heavy panels (hotspots, branches, recovery, graph…) load in the
  background with honest "Loading…" placeholders.
- Benchmark breadth: medium/merge-heavy/rename-heavy/long-history
  fixtures; file-lineage, branch-intelligence, and medium-repo FTS benches;
  `scripts/bench.sh` recording baselines to `benches/RESULTS.md`.
- Copy-source lineage: `Copied` changes in `gitx history --follow`/blame
  are followed to their source file; merge-touched files carry a
  `via_merge` marker.
- Installation docs: `cargo-dist` installers and a homebrew tap
  (docs/18 §9); `Cargo.lock` is committed and `--locked` builds verified.
- Schema v3 corrective migration: FTS5 delete/update triggers now use plain
  `DELETE FROM <fts> WHERE rowid = ...` (the FTS5 `'delete'` special command
  is only valid for contentless/external-content tables).
- TUI: animated loading/search spinner in the status bar; first-run
  onboarding hint (Overview banner until first navigation); cursor
  navigation (j/k moves the selection, Enter opens the row under it);
  "showing a–b of N" scroll-position indicator; log-scale repository-size
  gauge (small repos no longer render at 0%); high-contrast blue selection
  in the default theme; headless PTY verification harness
  (`scripts/verify-tui.sh`, 39 checks).

### Fixed
- TUI: Ctrl+C now always quits — it previously fell through to the view-jump
  arm in navigation mode (opening the Commits view) and could not quit while
  typing a search query.
- TUI Contributors: files-touched and top-areas looked up `author_lines` by
  raw author name, but the analysis keys it by `Name <email>` — areas now
  render for live analysis.
- Search previously queried FTS5 tables that migrations never created.
- Blame previously bailed; now a pure-gix line-level implementation.
- `gitx architecture diff` now matches the documented `architecture diff`
  subcommand shape.

## [0.1.0] - placeholder

### Added
- Initial scaffolding: 10-crate Rust workspace, migrations, tests, benches.
