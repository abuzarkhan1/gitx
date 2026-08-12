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
| 04-SYSTEM-ARCHITECTURE | ✅ | 10 crates + migrations/tests/benches populated; `gitx-services` now provides all six services + degraded index states (Indexed/PartiallyIndexed/Failed/Unsupported) surfaced in `status`/`info` |
| 05-DOMAIN-MODEL | ✅ | Entities present; identity normalization + config user-mapping (docs/05 §3) implemented and applied in `gitx contributors` |
| 06-DATABASE-SCHEMA | ✅ | v1–v3 migrations: all documented tables incl. FTS5 + corrected triggers |
| 07-CLI-SPECIFICATION | ✅ | Full surface + JSON + completions + exit codes 1–7; `release <TAG>`, `architecture --from/--to`, `search --since/--code`, blame `--limit` (docs/07 updated) |
| 08-TUI-SPECIFICATION | ✅ | All 14 views + drill-down + help overlay + `r`/`?` + small-terminal guard + background loading + **polish pass**: Overview charts (activity bar chart, language bars, health gauges, contributor bars, hotspot bars, repo-size gauge), full color hierarchy (severity/health/recency/diff colors, high-contrast selection, themed header+breadcrumb), plain-language explanations per metric + health verdict, consistent empty states, mouse support (wheel + sidebar click), view-jump keys (o/t/c/b/f/u/s/w/a/d/x/e/v, 1–9), sortable Hotspots (`s`), async FTS search with cursor + pending state, commit-graph timeline, ahead/behind bars, ownership bars, scroll-position indicator, `GITX_THEME`/`[ui] theme` |
| 09-INDEXING-ENGINE | ✅ | scan/refresh real; progress, Ctrl-C cancellation, atomic rebuild, corruption detection (exit 5), tag/reflog upsert, rewritten-history detection (HEAD-ref tracked, warning + meta flag) |
| 10-ANALYSIS-ENGINE | ✅ | + branch intelligence (ahead/behind/age/shared/merge-complexity), subsystem + knowledge + inactive ownership, release depth, risk/health formula+window, dependency usage + churn (`gitx dependencies usage`) |
| 11-SEARCH-SPECIFICATION | 🟡 | FTS5 + filters + JSON + `--since`/`--author` + `--code` + **TUI search now queries FTS across all scopes** (via `SearchService`); ranking is bm25-only |
| 12-RECOVERY-SPECIFICATION | ✅ | + `recovery export` patch, last-known-ref/reason/age presentation, GC warning, dangling trees/blobs (bounded reachability walk) |
| 13-PERFORMANCE-ENGINEERING | 🟡 | Bounded rayon walk (now fold/reduce — per-worker memory bounded, diffs not all held in RAM) + criterion benches (analysis, services) + index-backed stats/overview/analysis with live fallback; large-diff streaming still open |
| 14-TESTING-STRATEGY | ✅ | 83 tests incl. failure + property + edge-case + **services/consistency** (incremental-from-empty == full build == git count); `tests/fixtures/` documented with a deterministic `build.sh` |
| 15-SECURITY-PRIVACY | ✅ | Local-first and read-only; symlink traversal defended in the worktree code-search walker (Git-tree reads never touch the worktree) |
| 16-CONFIGURATION | ✅ | TOML + `config show|init` + `GITX_CONFIG`/`GITX_CACHE_DIR` env + repo `gitx.toml` layering + configurable cache dir |
| 17-IMPLEMENTATION-PLAN | ✅ | All phases landed (incl. graph compare, recovery, DX, distribution) |
| 18-RELEASE-ENGINEERING | ✅ | cargo-dist + workflow + checksums + release-check.sh + newer-schema detection (exit 5, explained message) |
| 19-QUALITY-GATES | ✅ | `scripts/check.sh` (fmt+clippy+check+test) + CI workflow with matrix |
| 20-ADR | ✅ | Decisions respected (e.g. analysis subdomains stay in `gitx-analysis`) |
| 21-ROADMAP | 🟡 | Stages 1–5 landed; Stage 6 (symbols/Tree-sitter/graphs) and Stage 7 polish pending |
| 22-CONTRIBUTING | ✅ | Docs only; §5 tracing requirement now met (see docs/03) |
| 23-FEATURE-MATRIX | ✅ | All feature rows done; “Persistent Index” column honored — stats/overview/hotspots/risk/health read the index with live fallback |
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

## Third implementation pass (five more workstreams, all verified)

Closed the remaining open audit items in five workstreams; **83 tests pass**,
clippy `-D warnings` clean, `cargo fmt` clean, `scripts/check.sh` green.

- **Index-backed analysis everywhere (docs/13 §3, docs/23)** — the
  analysis pipeline now persists per-file hotspot/classification metrics and
  health sub-scores into the index during `scan`/`refresh`/`rebuild`
  (`gitx-analysis::cache`); `gitx hotspots`, `gitx risk`, `gitx health`, and
  the TUI Hotspots/Health views read from a fresh index with live fallback
  (honest `is_current=0` rows for deleted files; `--no-cache` forces live).
  The "Persistent Index" column of docs/23 is now fully honored on the read
  path.
- **Application services + degraded states (docs/04 §6, §9)** — new
  `gitx-services` crate provides all six services (Repository, Index,
  Analysis, Search, Recovery, History); `gitx info`/`status`/`stats` and the
  index commands delegate to them; a `DegradedState` model
  (Indexed/PartiallyIndexed/Failed/Unsupported) is surfaced in `gitx
  status` and `gitx info`, each with an actionable remediation hint.
- **TUI depth (docs/08 §3)** — commit view renders parent-graph glyphs
  (`*` merge, `o` root) plus per-commit affected areas; file view adds
  hotspot score, classification, churn, author count, and ownership %;
  contributors view adds contribution weight (commits + churn), last-activity
  date, and touched-file count; architecture panel adds a module table with
  per-directory file count, churn, and modules added in the last 90 days.
- **TUI search hits SQLite FTS (docs/08 #20, docs/11)** — the `/` search
  path now queries the FTS index through `SearchService` across commits,
  files, authors, branches, and tags (with live fallback when the index is
  stale), instead of filtering the in-memory timeline.
- **Memory, benchmarks, consistency, symlinks (docs/13 §4/§8/§9, docs/14
  §8, docs/15 §5)** — the analysis pipeline now folds commit diffs
  chunk-by-chunk (`FileAccum` + `merge_file_acc`) so per-worker memory is
  bounded and diffs are no longer materialized as one `Vec<CommitDeltas>`;
  new criterion benches for `gitx-analysis` (health) and `gitx-services`
  (index build + search); a strict index-consistency test asserts
  incremental-from-empty == full build == `git rev-list` count; symlink
  defense confirmed for the only worktree walker (code search skips
  symlinks; every other scan reads Git trees and never touches the
  worktree).

## Fourth implementation pass (TUI usability + visuals, all verified)

Closed the TUI polish list (charts, colors, explanations, navigation, spec
markers, general polish) in five workstreams; **83 tests pass**, clippy
`-D warnings` clean, `cargo fmt` clean, `scripts/check.sh` green. Each
workstream was smoke-tested through a real PTY (alternate screen, keys sent,
frames captured and inspected).

- **Charts & visualizations (docs/08 §3 Overview)** — new `theme` module
  provides horizontal bars (`hbar`), vertical bar charts with week labels
  (`vchart`), and severity/health/recency colors. The Overview now renders:
  a real activity bar chart (12 weekly buckets with a label row), a
  language-breakdown bar list, six health gauges + overall gauge, top
  hotspots with score bars, a contributors-share bar list, and a
  repository-size gauge (small/medium/large). No bare numbers without a bar
  or context.
- **Color hierarchy (docs/25, docs/10 §2 bands)** — risk/hotspot scores
  are red/yellow/blue/green by band; classifications get badge colors;
  health sub-scores green/yellow/red; branch staleness green/yellow/red;
  commit-detail diff lines +green/−red; high-contrast selection
  (`sel_bg`); themed header with accent-colored branding, branch, state,
  and a “▸ current view” breadcrumb.
- **Explanations (docs/25 evidence-first)** — every Overview metric carries
  a one-line plain-language explanation; the Health view adds a verdict
  line (“mostly healthy — a few files may need attention”); every view
  shows a descriptive title (“Files — ranked by maintenance risk (0–100)”)
  and consistent empty states (“Run: gitx refresh …”) via `common::empty_rows`.
- **Navigation & discoverability (docs/08 #17/#18/#19/#20)** — mouse
  support enabled (scroll wheel scrolls lists; left-click on the sidebar
  jumps to a view); view-jump keys `o/t/c/b/f/u/s/w/a/d/x/e/v` and `1–9`;
  the Hotspots view is sortable (`s` cycles score/changes/churn); the FTS
  search runs on a worker thread with a visible cursor and a “Searching…”
  pending state (the UI never freezes); contextual status-bar hints per
  view plus a scroll-position indicator (“row x of N”).
- **Spec markers + polish (docs/08 #10/#14/#15, docs/16 §[ui])** — the
  Timeline renders a commit graph (lane glyphs `•`/`*`/`o` with
  `│` continuations); branches show ahead/behind bars; ownership shows
  per-file concentration bars; contributors show contribution-weight bars;
  loading shows progress in the status bar; themes are configurable via
  `GITX_THEME` or `[ui] theme` (default + light).
- **Drill-down depth (docs/08 #23, docs/10 §10)** — the Commits view adds a
  **related-commits panel**: selecting a commit lists up to three other
  commits touching overlapping files, ranked by shared-file count (computed
  from per-commit changed-file sets at load time); the Architecture view
  gains a **structural before/after comparison** — HEAD vs the newest
  commit ≥30 days old (falling back to the oldest), diffed with
  `gitx_graph::compare::compare_snapshots` — showing files added/removed/
  modified, added files, removed files, and modules (directories) that
  gained files; the file drill-down is upgraded from a plain history list
  to **rename-following lineage** — “Created by … on … — last change by …
  on …”, a per-action badge list (ADDED/MODIFIED/RENAMED from…/DELETED)
  with commit ids, dates, and messages (closes docs/08 §3 File view).

## Fifth implementation pass (headless PTY verification, all green)

Re-verified every polish item end-to-end by **driving the real TUI in a
tmux PTY** (alternate screen, keys + mouse SGR sequences sent, frames
captured and grepped — `scripts/verify-tui.sh`, 39 checks). This pass fixed
what the automated harness caught; **83 tests pass**, clippy `-D warnings`
clean, `cargo fmt` clean.

- **Cursor navigation (docs/08 #20)** — j/k now move the *selection* (`selected`),
  with the window scrolling only when the cursor leaves the visible area;
  Enter drills into the row under the cursor (previously the highlight and
  Enter were stuck on row 0 and j/k only scrolled). Verified: cursor
  highlight moves, Enter opens the exact commit under it, mouse wheel moves
  the cursor too, and the Health evidence panel follows the selected
  sub-score.
- **Loading spinner rendered** — `load_frame` was advanced on every tick but
  never drawn; the status bar now shows an animated `⣾⣽⣻⢿⡿⣟⣯⣷`
  spinner while the background loader or an FTS search runs.
- **First-run onboarding hint (docs/08 #31)** — the tracked-but-unused
  `nav_used` flag now drives a "Getting started — ↑↓ navigate · Enter open a
  view · / search · ? help" banner in the Overview that disappears after
  first navigation.
- **Repo-size gauge** — the linear 0–5000-file scale rendered small repos at
  ~0%; it now uses a log scale so small/medium/large all get a meaningful
  bar.
- **High-contrast selection** — the default theme's selection background
  changed from `DarkGray` (too subtle in long lists) to blue.
- **Ctrl+C always quits** — the `Ctrl-C` arm sat *after* the generic
  view-jump `Char(c)` arm, so Ctrl+C in navigation mode opened the Commits
  view instead of quitting; it is now handled first (and also during search
  input).
- **Contributor areas bug** — the Contributors view's files-touched and
  top-areas lookup matched on the raw author name, but the analysis keys
  `author_lines` by `Name <email>` (docs/05 identity normalization); the
  lookup now uses the full identity, so areas render for live analysis.
- **Scroll-position indicator** — upgraded from "row x of N" to a
  "showing a–b of N" range derived from the scroll offset and terminal
  height.
- **`scripts/verify-tui.sh`** — headless harness: builds a fixture repo,
  starts `gitx-tui` in tmux, drives every view via keys + mouse SGR
  sequences, captures frames, and asserts all 39 polish markers (charts,
  gauges, colors via 256-color escapes, hints, ranges, related-commits,
  lineage, sort, themes, Ctrl+C).

## Remaining gaps — complete line-by-line audit (docs ⇄ code)

Re-audited every doc word-by-word against the workspace. Items below are
either unimplemented or only partially implemented. Grouped by whether the
doc marks them MVP (P0/P1) or later (V1/V2/Later).

### MVP / P0–P1 gaps (specified as in-scope now)

Items marked **[closed]** were completed in the implementation passes below
(the first pass covered docs/07–08/10/09/12/14/15/16; the second pass covered
docs/13/05/03/12/18/14/25/08; the third pass covered docs/13/23/04/08/11/14/15;
the fourth pass covered the TUI usability/visual list — charts, colors,
explanations, mouse, quick keys, async search, themes, related-commits,
architecture before/after).

1. **[closed]** docs/03 §11, docs/22 §5 — Structured logging: `tracing` is
   wired end-to-end (`RUST_LOG` or `--verbose`, stderr only) with real log
   statements in the indexer, analysis pipeline, recovery scan, and repo
   discovery.
2. **[closed]** docs/04 §6 — Application services: new `gitx-services`
   crate provides all six services (Repository, Index, Analysis, Search,
   Recovery, History); the CLI and TUI delegate to them.
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
10. **[closed]** docs/08 §3 — Commit view: parent-graph glyphs, affected
    areas, and a related-commits panel (commits touching overlapping files,
    ranked by shared-file count) are now shown.
11. **[closed]** docs/08 §3 — File view: hotspot score, classification,
    churn, author count, and ownership % are shown in the list; the file
    drill-down (Enter) shows rename-following lineage with
    creation/last-change authors + dates and rename events.
12. **[closed]** docs/08 §3 — Branch view shows age + last activity per
    branch.
13. **[closed]** docs/08 §3 — Contributors view now shows contribution
    weight (commits + churn), last-activity date, and touched-file count
    per contributor.
14. **[closed]** docs/08 §3 — Timeline shows a changed-file count column;
    parent-graph glyphs landed in the commit view (docs/08 #10).
15. **[closed]** docs/08 §3 — Health sub-scores are selectable and reveal
    per-score evidence.
16. **[closed]** docs/08 §4 — `r` reloads data; `?` opens a help overlay.
17. **[closed]** docs/08 §5 — Responsive layout: small-terminal guard renders
    centered guidance below 60×20 instead of a mangled layout.
18. **[closed]** docs/08 §6 — Loading states: repository data loads on a
    background thread; the status bar shows progress while it computes.
19. **[closed]** docs/08 §7 — Empty states now include "Run: gitx refresh"
    guidance (commits/branches views).
20. **[closed]** docs/08 §3 Search, docs/11 — TUI search now queries the
    SQLite FTS index via `SearchService` across commits/files/authors/
    branches/tags (live fallback when the index is stale).
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
31. **[closed]** docs/13 §3, docs/23 “Persistent Index” column —
    Index-backed analysis: stats, overview, hotspots, risk, and health all
    read from a fresh persisted index (labeled `source: index`) with live
    fallback; deleted files are honestly reported. The sub-second-startup
    target for very large repos remains open (see item 54).
32. **[closed]** docs/13 §4/§8 — Memory: the pipeline folds commit diffs
    chunk-by-chunk (per-worker memory bounded; no `Vec<CommitDeltas>`);
    large-diff streaming is still open (moved to item 54).
33. **[closed]** docs/13 §9 — Benchmarks: new criterion benches for
    `gitx-analysis` (health) and `gitx-services` (index build + search);
    benchmark fixture repos are generated hermetically in the services bench.
34. **[closed]** docs/14 §3 — Integration coverage: new `edge_cases` test
    covers merge commits, binary files, empty commits, revert commits, and
    rewritten branches.
35. **[closed]** docs/14 §4 — Fixture repositories: `tests/fixtures/` now has
    a README and a deterministic `build.sh` reproducing the edge-case
    fixture; tests build fixtures hermetically at runtime.
36. **[closed]** docs/14 §6 — Property tests: identity normalization
    (idempotence, key stability, mapping safety) plus existing score/band
    properties.
37. **[closed]** docs/14 §8 — Index consistency: a strict test asserts
    incremental-from-empty == full rebuild == `git rev-list` count on the
    edge-case fixture.
38. **[closed]** docs/14 §9 — Failure tests: corrupted-index (exit 5),
    missing-`.git` (exit 4), invalid path, malformed config, invalid args.
39. **[closed]** docs/15 §5 — Symlink defense: the only worktree walker
    (code search) skips symlinks; every other scan reads Git trees and never
    touches the worktree, so no symlink can be followed outside the repo.
40. **[closed]** docs/16 §4 — Config precedence: defaults → global file →
    repo `gitx.toml` → env vars (`GITX_CONFIG`, `GITX_CACHE_DIR`).
41. **[closed]** docs/16 §6 — Configurable cache location: `[index]
    cache_dir` / `GITX_CACHE_DIR` wired into the index path.
42. **[closed]** docs/18 §7 — Newer-index detection: opening a newer-schema
    index is detected, explained, and exits 5 instead of being silently
    trusted.
43. **[closed]** docs/08 §3 Architecture — TUI graph/table: the panel shows
    a module table (file count, churn, modules added in the last 90 days)
    **and** a structural before/after comparison (HEAD vs the newest commit
    ≥30 days old) with files added/removed/modified and modules gained.
44. **[closed]** docs/04 §9 — Degraded states: Indexed/PartiallyIndexed/
    Failed/Unsupported model surfaced in `gitx status` and `gitx info` with
    remediation hints.
45. **[closed]** docs/10 §13 — Explainability contract: risk/health print the
    formula, weights, and the 30-day analysis window (docs/10 §13).
46. **docs/02 §4 V1 — Advanced filters / richer metrics.** *Partial:*
    filters exist for timeline/search and subsystem ownership is live
    (docs/10 §4); direct/indirect dependency classification and architecture
    milestones remain (see items 53, 54).
47. **[closed]** docs/25 §8 — Long output: `timeline` and `reflog` page
    through `less -R` on a TTY; non-TTY output prints directly.
48. **[closed]** docs/08 §3 Overview — Charts: activity bar chart with week
    labels, language bars, six health gauges + overall, top-hotspot score
    bars, contributor-share bars, and a repository-size gauge; every metric
    has a plain-language explanation and the health view adds a verdict.
49. **[closed]** docs/08, docs/25, docs/10 §2 — Colors: severity band
    colors (red/yellow/blue/green), classification badges, health
    green/yellow/red, staleness recency colors, +green/−red diff lines in
    the commit detail, high-contrast selection, and a themed header with a
    “▸ current view” breadcrumb.
50. **[closed]** docs/08 #17/#18 — Mouse support (scroll wheel + sidebar
    click) and view-jump keys (`o/t/c/b/f/u/s/w/a/d/x/e/v`, `1–9`); the
    Hotspots table is sortable (`s` cycles score/changes/churn).
51. **[closed]** docs/08 #20 — Async FTS search in the TUI: queries run on
    a worker thread with a visible cursor and a “Searching…” pending state;
    the status bar shows contextual hints and a scroll-position indicator.
52. **[closed]** docs/08 #10/#14/#15, docs/16 §[ui] — Timeline commit
    graph (`•`/`*`/`o` with `│` lanes), branch ahead/behind bars, ownership
    concentration bars, contributor-weight bars, and configurable themes
    (`GITX_THEME` / `[ui] theme`, default + light).

### Later / V1–V2 gaps (docs mark these as future work — listed for
completeness, not defects)

53. **docs/10 §10 — Architecture milestones and dependency-direction
    changes** are not detected (only added/removed/modified + new modules).
54. **docs/10 §11 — Dependency depth:** usage (files referencing a dep) and
    change-churn are live via `gitx dependencies usage`; direct/indirect
    classification, cargo features, and pnpm catalogs remain unresolved.
55. **docs/11 §8 — Ranking tiers.** bm25 relevance only; the exact >
    path/name > recent > textual priority tiers are not implemented
    (results are still deterministic).
56. **docs/21 Stage 6, docs/02 V1/V2 — Language symbols, Tree-sitter,
    structural graphs.** `gitx-graph::treesitter` is a `DummyParser` stub
    (ADR-011 Proposed); `CodeGraph` exists but no command/TUI consumes it;
    symbol history and language-aware analysis are unimplemented.
57. **docs/02 V2 — Advanced copy/rename lineage, richer export formats.**
58. **docs/18 §9 — Installation docs** cover binary download, source build,
    and completions; package-manager installation is still "later".
59. **docs/13 §4/§8 — Large-diff memory streaming** (incremental diff
    materialization and bounded output for `gitx diff` on huge commits) and
    the sub-second-startup target for very large repositories.

## Deliberate non-goals preserved

No AI, no network, no accounts: everything added is deterministic repository
analysis. All recovery paths are read-only.
