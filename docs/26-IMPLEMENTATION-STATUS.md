# Implementation Status — docs ⇄ code audit

Status of every specification in `docs/` against the Rust workspace, as of this
audit. Legend:

- ✅ **Implemented** — real, working code (not scaffolding)
- 🟡 **Partial** — core works, documented gaps remain
- ⬜ **Not started** — spec only

## Audit matrix

| Doc | Status | Notes |
| --- | --- | --- |
| 01-PRD | ✅ | Vision, goals, personas implemented across crates |
| 02-PRODUCT-SCOPE | ✅ | V1 features in; architecture diff, releases, lineage, regressions all live |
| 03-TECH-STACK | ✅ | All deps present; `tracing` wired with `RUST_LOG`/`--verbose` filters and real log statements (indexer, pipeline, recovery, discover) |
| 04-SYSTEM-ARCHITECTURE | 🟡 | 10 crates + migrations/tests/benches populated; only `HistoryService` of the six services |
| 05-DOMAIN-MODEL | ✅ | Entities present; identity normalization + config user-mapping (docs/05 §3) implemented and applied in `gitx contributors` |
| 06-DATABASE-SCHEMA | ✅ | v1–v3 migrations: all documented tables incl. FTS5 + corrected triggers |
| 07-CLI-SPECIFICATION | ✅ | Full surface + JSON + completions + exit codes 1–7; `release <TAG>`, `architecture --from/--to`, `search --since/--code`, blame `--limit` (docs/07 updated) |
| 08-TUI-SPECIFICATION | ✅ | All 14 views + drill-down + help overlay + working `r`/`?` + activity chart + health evidence + small-terminal guard (docs/08 §6) + background loading with status-bar progress |
| 09-INDEXING-ENGINE | ✅ | scan/refresh real; progress, Ctrl-C cancellation, atomic rebuild, corruption detection (exit 5), tag/reflog upsert, rewritten-history detection (HEAD-ref tracked, warning + meta flag) |
| 10-ANALYSIS-ENGINE | ✅ | + branch intelligence (ahead/behind/age/shared/merge-complexity), subsystem + knowledge + inactive ownership, release depth, risk/health formula+window, dependency usage + churn (`gitx dependencies usage`) |
| 11-SEARCH-SPECIFICATION | 🟡 | FTS5 + filters + JSON + working `--since`/`--author` (join fix) + `--code` worktree search; ranking is bm25-only |
| 12-RECOVERY-SPECIFICATION | ✅ | + `recovery export` patch, last-known-ref/reason/age presentation, GC warning, dangling trees/blobs (bounded reachability walk) |
| 13-PERFORMANCE-ENGINEERING | 🟡 | Bounded rayon walk + criterion bench + index-backed stats/overview with live fallback (docs/13 §3 partial: file-level analysis still computes live); full history in RAM, only one bench |
| 14-TESTING-STRATEGY | ✅ | 77 tests incl. failure + property + edge-case (merge/binary/empty/revert/rewrite) tests; `tests/fixtures/` now documented with a deterministic `build.sh` |
| 15-SECURITY-PRIVACY | 🟡 | Local-first and read-only; symlink traversal defended in code search (full scan defense pending) |
| 16-CONFIGURATION | ✅ | TOML + `config show|init` + `GITX_CONFIG`/`GITX_CACHE_DIR` env + repo `gitx.toml` layering + configurable cache dir |
| 17-IMPLEMENTATION-PLAN | ✅ | All phases landed (incl. graph compare, recovery, DX, distribution) |
| 18-RELEASE-ENGINEERING | ✅ | cargo-dist + workflow + checksums + release-check.sh + newer-schema detection (exit 5, explained message) |
| 19-QUALITY-GATES | ✅ | `scripts/check.sh` (fmt+clippy+check+test) + CI workflow with matrix |
| 20-ADR | ✅ | Decisions respected (e.g. analysis subdomains stay in `gitx-analysis`) |
| 21-ROADMAP | 🟡 | Stages 1–5 landed; Stage 6 (symbols/Tree-sitter/graphs) and Stage 7 polish pending |
| 22-CONTRIBUTING | ✅ | Docs only; §5 tracing requirement now met (see docs/03) |
| 23-FEATURE-MATRIX | 🟡 | All feature rows done; “Persistent Index” column partially honored (stats/overview read the index; file-level analysis still computed live) |
| 24-DATA-FLOW-AND-ALGORITHMS | ✅ | Timeline, blame, diff stats, recovery, lineage follow the documented algorithms |
| 25-UX-AND-OUTPUT-GUIDELINES | ✅ | Evidence-first output + formula/window lines + long-output pagination (`less -R` on TTY) + TUI loading progress in the status bar |

## What this audit completed

### Database & search (`gitx-storage`, `gitx-search`)
- Schema **v2 migration**: FTS5 virtual tables (`commits_fts`, `files_fts`,
  `authors_fts`, `branches_fts`, `tags_fts`) with sync triggers, plus derived
  tables (`file_renames`, `branch_commits`, `file_ownership`, `symbols`,
  `dependencies`, `dependency_events`, `hotspots`, `metrics`) — the search
  backend previously queried tables that migrations never created.
- Migration runner is idempotent and preserves v1 data; versioned SQL mirrored
  in `migrations/0001_initial.sql` + `migrations/0002_search_and_derived.sql`.
- Schema **v3 corrective migration** (`migrations/0003_fts_delete_triggers.sql`):
  the v2 FTS5 sync triggers used the `'delete'` special INSERT command, which
  is only valid for contentless/external-content tables and raised
  "SQL logic error" on our normal-content tables — so any DELETE on `commits`
  (e.g. the indexer's wipe/recover path) failed. v3 drops and recreates the
  delete/update triggers using `DELETE FROM <fts> WHERE rowid = ...`.
  Existing v2 indexes are repaired on next open (verified end-to-end via CLI).

### Git engine (`gitx-git`)
- **Real diff line counts** — insertions/deletions computed from blob diffs,
  with added/deleted directory expansion and rename/copy handling
  (previously hardcoded to 0).
- **Reflog reader** — `Repository::reflog()` via gix (`Reference::log_iter`).
- **Blob-at-path + file listing** helpers; object-kind and object-db iteration
  for recovery analysis.

### History (`gitx-history`)
- **Timeline filters** — author/since/until and a real diff-based path filter
  (the path filter previously matched everything).
- **Line-level blame** — pure gix implementation (no shelling out to
  `git blame`): forward line attribution over the file's history using
  `gix::diff::blob` (imara_diff) hunks, with first-parent merge handling.

### Analysis (`gitx-analysis`)
- **Recovery module** — reflog collection across refs and unreachable-commit
  detection from the object database (read-only, docs/12).
- **Analysis pipeline** — walks history, accumulates per-file metrics, derives
  hotspots (0–100 with LOW/MEDIUM/HIGH/CRITICAL bands), composite risk
  (normalized to /100 with evidence), ownership concentration, and the six
  sub-score repository health composite.
- **Repository stats** — shared by the CLI `stats` command and the TUI overview.

### CLI (`gitx-cli`)
- Replaced the parse-only stub with real commands: `info`, `status`, `stats`,
  `scan`, `refresh`, `index status|rebuild|clear`, `timeline`, `commit`,
  `history` (`--lines`), `blame`, `branches`, `branch`, `contributors`,
  `contributor`, `ownership`, `hotspots`, `architecture`, `dependencies`,
  `risk`, `health`, `search` (lazy FTS5 index), `recovery`, `unreachable`,
  `release`. `--json` on analytical commands; documented exit codes;
  evidence-first human output.

### TUI (`gitx-tui`)
- **Timeline and Hotspots panels** now render real data (commit list with
  dates/authors; hotspot scores with evidence), alongside the Overview panel
  (commits, contributors, files, branches, tags, age, languages, HEAD) — all
  loaded eagerly at startup.

### Indexing (docs/09, now real)
- `gitx-index`'s `Indexer` is wired to **concrete providers**: `GitProvider`
  for `gitx-git::Repository` and a SQLite `StorageProvider`/`Transaction` in
  `gitx-storage` (explicit BEGIN/COMMIT transactions; `is_commit_indexed` runs
  through the open transaction). `gitx scan` does a full build, `gitx refresh`
  is a true incremental pass that stops at already-indexed boundaries.
- The persisted index now lives in `<git_dir>/gitx/index.sqlite` so it never
  pollutes the worktree or the analysis itself (docs/16 §6).

### Configuration (docs/16) and completions (docs/07)
- TOML config loading in `gitx-core` (platform-appropriate default path, all
  sections per docs/16 §3, defaults when the file is missing/partial).
- `gitx config show` / `gitx config init`; analysis hotspot weights are read
  from `[analysis]` and threaded into the pipeline (`HotspotWeights`).
- `gitx completions <shell>` via clap_complete (bash/zsh/fish/powershell).

### Architecture diff (docs/07 §11, docs/10 §10)
- `gitx architecture diff <ref1> <ref2>` builds per-tree snapshots (path → blob
  oid) and compares them with `gitx_graph::compare::compare_snapshots`:
  added/removed/modified files plus newly added modules.
- `gitx branch <name>` now reports **ahead/behind/divergence vs the default
  branch** (docs/07 §8).

### CI (docs/18–19)
- `.github/workflows/ci.yml`: fmt + check + test + bench-compile on push/PR.

### Tests, benches, tooling
- 30+ unit tests across core/storage/git/history/analysis/cli and **11
  integration tests** against real Git fixture repositories — including two
  end-to-end incremental-indexer tests (docs/14 layout now populated).
- Criterion micro-benchmark in `gitx-analysis/benches` (docs/13).
- `scripts/check.sh` (fmt + check + test) and `benches/README.md`.

### TUI panels (docs/08 — now all live)
- Every navigation panel renders real data: **Overview**, **Timeline**,
  **Commits** (with parents), **Branches** (local/remote + tip), **Files**
  (metrics), **Contributors** (commit counts), **Hotspots**, **Ownership**
  (concentration + top owner), **Architecture** (modules by file count),
  **Dependencies** (HEAD manifests), **Risk** (evidence per file),
  **Health** (six sub-scores), **Recovery** (reflog + unreachable, with a
  capped object scan so startup stays fast), and **Search** (`/` opens a
  query box that filters the loaded timeline live).

### Dependency engine (docs/10 §11)
- Manifest parsing moved into a shared `gitx_analysis::manifest` module used
  by both CLI and TUI: `Cargo.toml`, `package.json`, `go.mod`,
  `requirements.txt`, `pyproject.toml` (deterministic, heuristic).
- `gitx dependencies history` walks the mainline oldest→newest and reports
  **added/removed/version-changed** events per manifest (docs/10 §11
  dependency history).
- `gitx dependencies diff <REF1> <REF2>` reports the dependency delta between
  two refs (branch, tag, or commit id).

### Releases (docs/07 §17)
- `gitx release show <TAG>` (was `gitx release <TAG>`) plus **`gitx release
  diff <REF1> <REF2>`**: commits added across the window, files changed, and
  aggregate insertions/deletions.

### Release engineering (docs/18) — now wired up
- **cargo-dist config** in the root `Cargo.toml` (`[workspace.metadata.dist]`):
  four targets (macOS arm64/x86_64, Linux, Windows), `gitx` + `gitx-tui`
  binaries, shell/homebrew installers, uploads on tagged releases.
- **`.github/workflows/release.yml`**: preflight gate (fmt + clippy -D warnings
  + tests + bench compile) → per-platform release builds → SHA-256
  checksums → GitHub Release with notes from `CHANGELOG.md` (docs/18 §5:
  executable + checksum + release notes).
- **`.github/workflows/ci.yml`** expanded: clippy `-D warnings` (docs/19 §1),
  cross-platform build matrix (docs/18 §3/§6), and an end-to-end CLI smoke
  test against a real repo.
- **`CHANGELOG.md`** and **`docs/27-RELEASING.md`** (process guide: versioning,
  checklist, local `cargo dist plan` dry run).

### Dependency analysis depth (docs/10 §11)
- Lockfile-precise parsing: `Cargo.lock`, `package-lock.json`, `yarn.lock`,
  `pnpm-lock.yaml`, `go.sum` (`gitx_analysis::manifest::parse_lockfile` +
  `lockfile_dependencies_at`), each with unit tests.
- `gitx dependencies` (list) and `gitx dependencies diff` now merge declared
  constraints **and** resolved lockfile versions.

### TUI interactivity (docs/08 §5) — now with drill-down
- Panels are **scrollable and selectable**: Enter opens a view, j/k scrolls
  (with a highlighted selection), Esc/← returns to sidebar navigation; the
  status bar reflects the current mode. Timeline, Commits, Branches, Files,
  Contributors, Hotspots, Ownership, Dependencies, Risk, Recovery and Search
  results all render as ratatui lists.
- **Drill-down**: Enter on a Timeline/Commits/Search row opens the full commit
  (metadata, message, changed files with diff stats — mirrors `gitx commit`);
  Enter on a Files/Hotspots/Risk/Ownership row opens that file's history
  (mirrors `gitx history <path>`). Esc/← closes back to the originating panel.

### Release verification (docs/18 §5–§6)
- **`scripts/release-check.sh`**: builds optimized binaries for the current
  host, stages `gitx` + `gitx-tui`, generates SHA-256 checksums, smoke-tests
  20 CLI commands against a real repository, and verifies checksum integrity.
  Passed end-to-end locally; CI runs the same steps per platform.

### File lineage (docs/10 file archaeology)
- `gitx-history::lineage` rewritten from a placeholder into **real
  rename-following lineage**: walking the mainline backward, a commit is
  `Renamed` when the path matches a rename destination (with the original
  source recorded), otherwise `Modified`, and the traversal continues on the
  source path so the full life of the file is recovered.
- Fixing lineage exposed a real bug: `track_rewrites(None)` in `diff.rs`
  **silently disabled rename detection everywhere** (gix treats `None` as
  "no rewrite tracking"). It now uses `Rewrites::default()`, so `gitx
  lineage`, the indexer's `file_renames`, and architecture snapshots all see
  renames. New integration test locks rename-following in.
- New `gitx lineage <path>` command (docs/07 command surface).

### Bug/regression history (docs/10 §9)
- New `gitx_analysis::regressions` module + **`gitx regressions`** command:
  classifies commits (fix/revert/build) from messages, links `revert <oid>`
  mentions to their targets (abbreviated oid resolved against the object
  db), and reports per-path **fix density**, recurring fix areas, and
  revert-after-change patterns. Verified against the fixture.

### Workspace-aware dependency resolution (docs/10 §11)
- `gitx_analysis::manifest::detect_workspace` recognizes npm (`workspaces`),
  cargo (`[workspace]`) and pnpm (`pnpm-workspace.yaml`) monorepos; new
  `gitx dependencies workspace` command shows root + members.
- `parse_package_json` now handles **single-line JSON** (common in monorepo
  member manifests) without leaking root-level keys into the deps section.

### Parallel analysis (docs/13 §6)
- The analysis walk now runs on a **bounded rayon pool** (≤8 workers, named
  threads): commit diffs and per-file blob reads are the independent,
  CPU-heavy work, parallelized via `par_chunks` with a gix handle opened per
  chunk (the supported pattern for gix's Send-but-not-Sync `Repository`).
  Results are folded back in original order, so output is **bit-for-bit
  deterministic** — verified by a new integration test that analyzes the
  same fixture twice and compares every metric and health sub-score.

### Snapshot tests (docs/14 §5)
- New `tests/snapshots/*.snap` files + `cli_snapshots` integration test that
  runs the real binary against fixtures and compares normalized output
  (commit oids, ISO timestamps, durations, and the fixture's temp path are
  masked). Regenerate deliberately with `GITX_BLESS=1`.

## Implementation pass (five workstreams, all verified)

All MVP/P0–P1 gaps from the audit below were implemented and verified in one
pass (docs/07, docs/08, docs/10, docs/09, docs/12, docs/14, docs/15, docs/16):

- **CLI (docs/07)** — exit codes 5/6/7; `gitx release <TAG>` shorthand;
  `gitx architecture --from/--to`; `gitx search --since`; `gitx search
  --author` fixed (was a broken `author =` column on an FTS table — now a
  proper JOIN on commits/authors); `gitx search --code` (bounded worktree
  content search with binary/symlink/`.git` guards); `gitx blame --limit`
  pagination. Spec docs updated to match.
- **TUI (docs/08)** — working `r` (reload) and `?` (help overlay) keys;
  real header (repo/branch/state); Overview activity chart + top hotspots +
  recent commits + state; Timeline changed-file-count column; Branches
  age/activity columns; Health sub-scores are selectable and reveal per-score
  evidence; empty-state guidance ("Run: gitx refresh").
- **Branch/ownership/release (docs/10)** — real `branch_intelligence`
  (ahead/behind from reachability, age, shared files, labeled merge-complexity
  estimate, stale flag) wired into `gitx branches` and `gitx branch`;
  ownership gains subsystem (per-directory), knowledge-concentration, and
  inactive-ownership sections; `release diff` now reports contributors,
  classifications, top areas, and top hotspots touched.
- **Indexing + recovery (docs/09, docs/12)** — progress on stderr during
  scan/refresh; Ctrl-C cancellation (transaction rolled back); `index rebuild`
  is now atomic (temp build → validate → swap); `index status` detects
  corruption and exits 5; refresh upserts tags and reflog entries; `gitx
  recovery export <OID>` writes a real `git apply`-able unified patch;
  unreachable output shows age, last-known reference, reason, and the GC
  warning.
- **Config/security/explainability/tests (docs/14, 15, 16)** —
  `GITX_CONFIG` env var, repository `gitx.toml` layering (defaults → global →
  repo → env → CLI), configurable cache dir (`[index] cache_dir` /
  `GITX_CACHE_DIR`); symlink traversal blocked in code search; risk/health
  output now carries formula + time window + bands; new failure tests
  (corrupt index → exit 5, missing `.git` → exit 4, bad path, malformed
  config, invalid args) and property tests (scores/ownership bounded 0–100,
  bands partition the range).

64 tests pass, clippy `-D warnings` clean, `cargo fmt` clean.

## Second implementation pass (five more workstreams, all verified)

Closed the remaining open audit items in five workstreams; **77 tests pass**,
clippy `-D warnings` clean, `cargo fmt` clean, `scripts/check.sh` green.

- **Index-backed analysis (docs/13 §3, docs/09 §5)** — `gitx stats` and the
  TUI Overview read from the persisted SQLite index when it is fresh (HEAD
  matches `last_head` metadata), falling back to live Git analysis; stats
  source is labeled (`source: index`/`live`). Index refresh now tracks the
  symbolic HEAD ref (via `head_ref_name`) and flags rewritten history
  (force-push/rebase) in `index_metadata.rewritten_detected`, warning the
  user to rebuild.
- **Identity normalization (docs/05 §3)** — new `gitx-core::identity`
  module: conservative normalization (lowercased-email canonical key, never
  weak merges) plus explicit `[identity.mappings]` user mappings; `gitx
  contributors` groups by canonical identity and applies display-name
  mappings.
- **Recovery + dependencies depth (docs/12 §6, docs/10 §11)** — `gitx
  unreachable` now also reports dangling trees/blobs (bounded reachability
  walk over refs' commit trees via a new `tree_oids` helper); `gitx
  dependencies usage` reports files-referencing + change-churn per dependency
  (whole-word source scan over HEAD).
- **Observability + UX (docs/03 §11, docs/08 §6, docs/25)** — `tracing` is
  wired end-to-end (`RUST_LOG` or `--verbose`, stderr only) with real log
  statements in the indexer, analysis pipeline, recovery scan, and repo
  discovery; TUI renders a small-terminal guard below 60×20 and loads repo
  data on a background thread with status-bar progress; long CLI lists
  (`timeline`, `reflog`) page through `less -R` on a TTY.
- **Tests + release safety (docs/14 §5/§9, docs/18 §7)** —
  `tests/fixtures/` documented with a deterministic `build.sh` (merge,
  binary, empty, revert, rewrite fixture); new `edge_cases` integration test
  (5 tests) covering merge commits, binary diffs, empty commits, revert
  classification, and rewritten-history detection; identity property tests
  (idempotence, key stability, mapping safety); newer-schema detection — a
  valid index written by a newer build is explained and exits 5 instead of
  being silently trusted.

## Remaining gaps — complete line-by-line audit (docs ⇄ code)

Re-audited every doc word-by-word against the workspace. Items below are
either unimplemented or only partially implemented. Grouped by whether the
doc marks them MVP (P0/P1) or later (V1/V2/Later).

### MVP / P0–P1 gaps (specified as in-scope now)

Items marked **[closed]** were completed in the implementation passes below
(the first pass covered docs/07–08/10/09/12/14/15/16; the second pass covered
docs/13/05/03/12/18/14/25/08).

1. **[closed]** docs/03 §11, docs/22 §5 — Structured logging: `tracing` is
   wired end-to-end (`RUST_LOG` or `--verbose`, stderr only) with real log
   statements in the indexer, analysis pipeline, recovery scan, and repo
   discovery.
2. **docs/04 §6 — Application services.** Only `HistoryService` exists.
   `RepositoryService`, `IndexService`, `AnalysisService`, `SearchService`,
   `RecoveryService` are missing — business logic lives in CLI command
   functions instead of the documented service layer.
3. **[closed]** docs/05 §3 — Identity normalization: raw + normalized display
   identity and explicit `[identity.mappings]` user mappings in
   `gitx-core::identity`, applied in `gitx contributors` (canonical key =
   lowercased email; no weak merges).
4. **[closed]** docs/07 §17 — `gitx release <TAG>` shorthand implemented;
   docs/07 updated.
5. **[closed]** docs/07 §11 — `gitx architecture --from/--to` implemented;
   docs/07 updated.
6. **[closed]** docs/07 §19 — Exit codes 5/6/7 mapped; docs/07 updated.
7. **[closed]** docs/07 §7, docs/24 §13 — `gitx blame --limit` (default 500)
   pagination; `history --lines` uses the same bound.
8. **[closed]** docs/07 §8, docs/10 §5 — Full `branch_intelligence`
   (ahead/behind/age/shared files/merge-complexity estimate/stale) wired
   into `gitx branches` and `gitx branch`.
9. **[closed]** docs/08 §3 — Overview now shows activity chart, top
   hotspots, recent commits, state, live branch/name header.
10. **docs/08 §3 — Commit view.** Missing: related-commits, affected-areas,
    parent-graph panels.
11. **docs/08 §3 — File view.** Missing: lineage, first/last change, rename
    events, hotspot metrics, churn (drill-down shows raw history only).
12. **[closed]** docs/08 §3 — Branch view shows age + last activity per
    branch.
13. **docs/08 §3 — Contributors view.** Commit counts only; missing files,
    contribution weight, areas, ownership concentration.
14. **[closed]** docs/08 §3 — Timeline now shows a changed-file count column
    (commit graph glyphs still pending).
15. **[closed]** docs/08 §3 — Health sub-scores are selectable and reveal
    per-score evidence.
16. **[closed]** docs/08 §4 — `r` reloads data; `?` opens a help overlay.
17. **[closed]** docs/08 §5 — Responsive layout: small-terminal guard renders
    centered guidance below 60×20 instead of a mangled layout.
18. **[closed]** docs/08 §6 — Loading states: repository data loads on a
    background thread; the status bar shows progress while it computes.
19. **[closed]** docs/08 §7 — Empty states now include "Run: gitx refresh"
    guidance (commits/branches views).
20. **docs/08 §3 Search, docs/11 — TUI search** filters the in-memory
    timeline only; it does not query FTS across commits/files/authors/
    branches/tags as the spec requires.
21. **[closed]** docs/09 §7 — Cancellation: Ctrl-C aborts scan/refresh and
    rolls the open transaction back.
22. **[closed]** docs/09 §8 — Progress: `ConsoleProgress` reports every 1000
    commits on stderr; stdout stays machine-clean.
23. **[closed]** docs/09 §9 — Atomic rebuild: temp-index → validate → swap.
24. **[closed]** docs/09 §10 — Corruption recovery: corrupt index is detected
    and reported with exit 5 and a rebuild hint.
25. **[closed]** docs/09 §3/§4 — Incremental detection: refresh detects
    branch moves, deleted refs, tag/reflog changes, and rewritten history
    (HEAD-ref tracked; `rewritten_detected` meta + warning).
26. **[closed]** docs/10 §4 — Ownership: subsystem (per-directory),
    knowledge-concentration, and inactive-ownership sections added.
27. **[closed]** docs/10 §5 — Merge complexity / shared files computed and
    shown by `gitx branch`.
28. **[closed]** docs/10 §12 — Release analysis: `release diff` reports
    contributors, classifications, top areas, and top hotspots.
29. **[closed]** docs/11 §4 — `gitx search --since` implemented (date-only
    parsing); `--author` join fixed.
30. **[closed]** docs/12 §2/§5/§6 — Recovery depth: dangling trees/blobs
    (bounded reachability walk), last-known-reference + reason + age,
    `recovery export` patch action, and GC-pruning warning.
31. **docs/13 §3, docs/23 “Persistent Index” column — Index-backed analysis.**
    *Partial:* `gitx stats` and the TUI Overview read from a fresh persisted
    index (labeled `source: index`) with live fallback; file-level analysis
    (hotspots/risk/health) still computes live because the incremental
    indexer does not populate `file_changes`. Full index-backed analysis and
    the sub-second-startup target remain open.
32. **docs/13 §4/§8 — Memory.** The analysis pipeline materializes every
    commit diff in RAM (`Vec<CommitDeltas>`); long CLI lists are paged via
    `less -R` on a TTY, but large diffs are not streamed.
33. **docs/13 §9 — Benchmarks.** Only one criterion bench (hotspots). No
    benchmark fixture repos and no benches for index, refresh, search, branch
    analysis, file history, or TUI prep.
34. **[closed]** docs/14 §3 — Integration coverage: new `edge_cases` test
    covers merge commits, binary files, empty commits, revert commits, and
    rewritten branches.
35. **[closed]** docs/14 §4 — Fixture repositories: `tests/fixtures/` now has
    a README and a deterministic `build.sh` reproducing the edge-case
    fixture; tests build fixtures hermetically at runtime.
36. **[closed]** docs/14 §6 — Property tests: identity normalization
    (idempotence, key stability, mapping safety) plus existing score/band
    properties.
37. **docs/14 §8 — Index consistency.** A convergence test exists but there is
    no strict assertion that a fresh full index equals an incremental index
    built from empty.
38. **[closed]** docs/14 §9 — Failure tests: corrupted-index (exit 5),
    missing-`.git` (exit 4), invalid path, malformed config, invalid args.
39. **docs/15 §5 — Symlink defense.** Code-search traversal skips symlinks;
    the full repository-traversal scan is not symlink-defended.
40. **[closed]** docs/16 §4 — Config precedence: defaults → global file →
    repo `gitx.toml` → env vars (`GITX_CONFIG`, `GITX_CACHE_DIR`).
41. **[closed]** docs/16 §6 — Configurable cache location: `[index]
    cache_dir` / `GITX_CACHE_DIR` wired into the index path.
42. **[closed]** docs/18 §7 — Newer-index detection: opening a newer-schema
    index is detected, explained, and exits 5 instead of being silently
    trusted.
43. **docs/08 §3 Architecture — TUI graph/table.** The panel shows current
    modules only; no structural graph/table comparison (docs/08 "graph or
    table depending on terminal dimensions").
44. **docs/04 §9 — Degraded states.** No Indexed/PartiallyIndexed/Failed/
    Unsupported state model is surfaced anywhere.
45. **[closed]** docs/10 §13 — Explainability contract: risk/health print the
    formula, weights, and the 30-day analysis window (docs/10 §13).
46. **docs/02 §4 V1 — Advanced filters / richer metrics.** Partial: filters
    exist for timeline/search; direct/indirect dependency classification and
    subsystem ownership are absent (see items 26, 48).
47. **[closed]** docs/25 §8 — Long output: `timeline` and `reflog` page
    through `less -R` on a TTY; non-TTY output prints directly.

### Later / V1–V2 gaps (docs mark these as future work — listed for
completeness, not defects)

48. **docs/10 §10 — Architecture milestones and dependency-direction
    changes** are not detected (only added/removed/modified + new modules).
49. **docs/10 §11 — Dependency depth:** usage (files referencing a dep) and
    change-churn are live via `gitx dependencies usage`; direct/indirect
    classification, cargo features, and pnpm catalogs remain unresolved.
50. **docs/11 §8 — Ranking tiers.** bm25 relevance only; the exact >
    path/name > recent > textual priority tiers are not implemented
    (results are still deterministic).
51. **docs/21 Stage 6, docs/02 V1/V2 — Language symbols, Tree-sitter,
    structural graphs.** `gitx-graph::treesitter` is a `DummyParser` stub
    (ADR-011 Proposed); `CodeGraph` exists but no command/TUI consumes it;
    symbol history and language-aware analysis are unimplemented.
52. **docs/02 V2 — Advanced copy/rename lineage, richer export formats.**
53. **docs/18 §9 — Installation docs** cover binary download, source build,
    and completions; package-manager installation is still "later".

## Deliberate non-goals preserved

No AI, no network, no accounts: everything added is deterministic repository
analysis. All recovery paths are read-only.
