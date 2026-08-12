# Changelog

All notable changes to GitX are documented here. Format follows
[Keep a Changelog](https://keepachangelog.com/); versions follow
[Semantic Versioning](https://semver.org/).

## [0.1.0] - 2026-08-13

First public release: a local-first, terminal-native Git repository
intelligence and code archaeology tool.

### Added
- SQLite history index with incremental refresh (`gitx scan` / `gitx refresh`)
  and atomic rebuild, corruption detection, and Ctrl-C cancellation.
- Timeline, commit detail, file history with rename/copy lineage, and
  line-level blame.
- Deterministic intelligence: hotspots, per-file risk, composite repository
  health (six sub-scores), ownership concentration, branch intelligence,
  and regression/fix-density analysis.
- Dependency analysis across manifests and lockfiles (Cargo, npm/yarn/pnpm,
  Go), workspace detection, and dependency usage/churn.
- Full-text search over commits, files, authors, branches, tags, symbols,
  directories, renames, and code content.
- Recovery: reflog inspection, unreachable commits, dangling trees/blobs,
  and `recovery export` patches.
- Architecture: directory evolution, structural diffs, milestones, and a
  heuristic code graph with call edges.
- Interactive TUI (`gitx-tui`) with 15 views, drill-down, themes, mouse
  support, and phased lazy loading.
- JSON and CSV output on all analytical commands; shell completions; TOML
  configuration; `cargo-dist` installers (shell/powershell/homebrew).

### Fixed
- FTS5 delete/update triggers (schema v3 migration).
- TUI Ctrl+C quitting; contributor areas keyed by canonical identity;
  architecture diff subcommand shape.

The full feature matrix lives in `docs/23-FEATURE-MATRIX.md`.
