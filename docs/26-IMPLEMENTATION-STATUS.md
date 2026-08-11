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
| 02-PRODUCT-SCOPE | 🟡 | V1 features in; `gitx architecture diff`, releases remain |
| 03-TECH-STACK | 🟡 | All deps used now (rayon, ignore, criterion wired); cargo-dist not configured |
| 04-SYSTEM-ARCHITECTURE | ✅ | 10 crates + migrations/tests/benches/scripts populated |
| 05-DOMAIN-MODEL | ✅ | Commit, Branch, Tag, ObjectId, FileChange, derived models |
| 06-DATABASE-SCHEMA | ✅ | v1+v2 migrations: all documented tables incl. FTS5 + triggers |
| 07-CLI-SPECIFICATION | 🟡 | All core commands implemented with JSON + exit codes; completions missing |
| 08-TUI-SPECIFICATION | 🟡 | 14-view nav + real Overview data; other views placeholder |
| 09-INDEXING-ENGINE | 🟡 | `gitx scan`/`refresh` build the full index; incremental Indexer awaits provider impls |
| 10-ANALYSIS-ENGINE | ✅ | Hotspots, risk, ownership, health, classification, pipeline |
| 11-SEARCH-SPECIFICATION | ✅ | FTS5 backend over commits/files/authors/branches/tags |
| 12-RECOVERY-SPECIFICATION | ✅ | Reflog reading + unreachable-commit detection (read-only) |
| 13-PERFORMANCE-ENGINEERING | 🟡 | Criterion bench + batching; no rayon in the analysis walk yet |
| 14-TESTING-STRATEGY | 🟡 | 15+ unit tests + 8 integration tests with real fixtures; snapshot tests missing |
| 15-SECURITY-PRIVACY | ✅ | Everything local-first and read-only by construction |
| 16-CONFIGURATION | ⬜ | `--config` parsed; config file loading not implemented |
| 17-IMPLEMENTATION-PLAN | 🟡 | Phases 1–4, 6, 8 substantially done; 5 (graph) and 7 (DX) partial |
| 18-RELEASE-ENGINEERING | ⬜ | No cargo-dist config, no CI workflow |
| 19-QUALITY-GATES | 🟡 | `scripts/check.sh` (fmt+check+test); CI not configured |
| 20-ADR | ✅ | Decisions respected (e.g. analysis subdomains stay in `gitx-analysis`) |
| 21-ROADMAP | 🟡 | Stages 1–2 landed; 3–5 partial |
| 22-CONTRIBUTING | ✅ | Docs only, no code required |
| 23-FEATURE-MATRIX | 🟡 | Matrix rows implemented for V1 commands; TUI panels pending |
| 24-DATA-FLOW-AND-ALGORITHMS | ✅ | Timeline, blame, diff stats, recovery all follow the documented algorithms |
| 25-UX-AND-OUTPUT-GUIDELINES | 🟡 | Evidence-first risk/health output done; full TUI UX pending |

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

## Remaining gaps (honest list)

1. **Per-ecosystem resolution semantics** — workspace-aware and
   feature-aware resolution (npm workspaces, cargo features, pnpm catalogs)
   beyond the flat lockfile extraction is future work.
2. **Release automation verification** — the workflows and local verification
   script are in place; a real tagged release end-to-end run is pending a
   public repository.
4. **Snapshot tests** (docs/14) and rayon-parallel analysis (docs/13) remain
   future work; the analysis walk is currently sequential.

## Deliberate non-goals preserved

No AI, no network, no accounts: everything added is deterministic repository
analysis. All recovery paths are read-only.
