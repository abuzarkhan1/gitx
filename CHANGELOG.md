# Changelog

All notable changes to GitX are documented here. Format follows
[Keep a Changelog](https://keepachangelog.com/); versions follow
[Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added
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
- Schema v3 corrective migration: FTS5 delete/update triggers now use plain
  `DELETE FROM <fts> WHERE rowid = ...` (the FTS5 `'delete'` special command
  is only valid for contentless/external-content tables).

### Fixed
- Search previously queried FTS5 tables that migrations never created.
- Blame previously bailed; now a pure-gix line-level implementation.
- `gitx architecture diff` now matches the documented `architecture diff`
  subcommand shape.

## [0.1.0] - placeholder

### Added
- Initial scaffolding: 10-crate Rust workspace, migrations, tests, benches.
