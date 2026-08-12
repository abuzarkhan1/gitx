# Remaining Work Implementation Plan (post sixth-pass)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close every remaining docs-vs-code gap (symbol history, language-aware complexity, structural-graph depth, advanced filters, `gitx diff` streaming, CSV export, lazy TUI startup, benchmarks, install docs, lineage depth) and land the in-flight sixth implementation pass.

**Architecture:** Work is organized into four independently executable workstreams so any subset can be cut without breaking the rest: (A) analysis intelligence depth, (B) CLI breadth, (C) TUI + performance, (D) docs/release/hygiene. Analysis subdomains stay in `gitx-analysis` (ADR convention); graph primitives stay in `gitx-graph`; the CLI and TUI delegate through `gitx-services` where a service already exists. Task 0 first lands the uncommitted sixth pass so later tasks start from a clean, committed tree.

**Tech Stack:** Rust workspace (11 crates), gix 0.66, rusqlite 0.32 (bundled), clap 4.5, ratatui 0.28, crossterm 0.28, rayon, petgraph. No new dependencies anywhere in this plan (the CSV writer is hand-rolled; Tree-sitter remains deferred per ADR-011).

## Global Constraints

- **No new dependencies.** Use only crates already in the workspace `Cargo.toml`.
- **Deterministic output.** Every new command/score must be bit-for-bit reproducible (parallel work folds in original order; tests assert exact values or use bounded-range properties).
- **Read-only analysis.** New code reads Git trees and object DBs only; never the worktree (except the already-bounded code search).
- **Analysis subdomains live in `gitx-analysis`.** Symbol extraction, complexity, and symbol history belong there, not in `gitx-graph` or `gitx-cli`.
- **Evidence-first output (docs/25).** Any new metric prints its formula/source; a complexity fallback must be labeled (`loc` vs `symbols+loc`), never silently zeroed (docs/10 §2).
- **Quality gates before every commit:** `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`. Run `scripts/check.sh` (does all three plus tests). After TUI changes, `scripts/verify-tui.sh` must stay green (currently 39/39).
- **Snapshot regeneration is deliberate.** Never re-bless snapshots to hide a bug; `GITX_BLESS=1` only after manually reviewing the diff.
- **Commit style.** Follow the repo convention from `git log`: `feat(scope): short imperative summary`. Never stage `.freebuff/` database files.
- **Fixture pattern.** Integration tests build repositories hermetically at runtime (see `tests/integration/lineage.rs` and `tests/fixtures/build.sh`); never depend on the workspace repo itself.
- **Docs stay truthful.** Every doc claim must match code; update `docs/26-IMPLEMENTATION-STATUS.md` and `docs/23-FEATURE-MATRIX.md` as tasks land.

---

## Audit summary (what remains, and why)

Verified against the working tree on 2026-08-12 (`cargo build --workspace` clean, `cargo test --workspace` green, 83+ tests). The sixth implementation pass (tiered ranking, `--renames`/`--history`, symbol + directory search scopes, `architecture milestones`, `dependencies features`, `gitx symbols`, `gitx graph`, dependency-direction flips, copy lineage, TUI polish) is **written but uncommitted** — 33 modified files + 3 new modules. Everything else documented as ✅ in `docs/26` is committed and verified.

Open items, mapped to tasks:

| # | Gap (doc reference) | Current state | Task |
|---|---|---|---|
| 1 | Sixth pass uncommitted | 2075 insertions in working tree | Task 0 |
| 2 | Complexity signal is LOC-only; docs/10 §2 wants heuristic function count and no silent zeros | `n_complexity = scale(loc, max_loc)` in `pipeline.rs:294`; `function_count` extractor exists but is unused for scoring | Task 1 |
| 3 | No symbol history (docs/21 Stage 6) | `symbols` table is HEAD-only | Task 2 |
| 4 | Dead `gitx_graph::dependency` module; `gitx graph` has no call edges; no TUI graph view (docs/02 V1 "stronger architecture graph") | `dependency.rs` unconsumed; `graph` CLI builds only Contains + Imports | Task 3 |
| 5 | Advanced filters (docs/02 V1) | timeline lacks `--committer`/`--merges`/`--no-merges`; search lacks `--until`/`--path` | Task 4 |
| 6 | No `gitx diff`; docs/13 §4/§8 want streamed, bounded diff output | only `release diff` / `architecture diff` / commit patches exist | Task 5 |
| 7 | Richer export formats (docs/02 V2) | `--json` only | Task 6 |
| 8 | Sub-second startup / lazy TUI loading (docs/13 §3/§7) | `load_repo_stats` loads every panel eagerly in one pass | Task 7 |
| 9 | Benchmark breadth + regression gate (docs/13 §9/§10) | benches cover analysis + services only | Task 8 |
| 10 | ADR-011 Tree-sitter status stale; `gitx-graph::treesitter` is a no-op stub nothing consumes | ADR-011 "Proposed"; docs/23 claims a parser placeholder exists | Task 9 |
| 11 | Package-manager installation is a "later" placeholder (docs/18 §9) | docs/18 §9 says "planned but not yet published" | Task 10 |
| 12 | Copy detection only at file birth; no merge marker in lineage (docs/02 V2) | `copy_source` used only in the root/A dded arms; `Copied` change type falls through | Task 11 |
| 13 | Repo hygiene: `.freebuff/` DB tracked (8 MB churn), scratch files committed, `Cargo.lock` ignored despite `--locked` installs | `git ls-files` shows `.freebuff/*`, `test_gix.rs`, `fix_repo.sh`; `.gitignore` ignores `Cargo.lock` | Task 12 |

Deliberately **not** planned (documented decisions): AI/network/accounts (non-goals), a real Tree-sitter adapter (Task 9 defers it with rationale — the heuristic extractor already satisfies the docs/23 matrix), and cross-branch lineage (mainline-first is the documented model in docs/10).

---

### Task 0: Land the sixth implementation pass

The working tree contains the finished sixth pass (search tiers, symbols, structure milestones, dependency features, TUI polish) plus docs/26, docs/23, CHANGELOG updates. Nothing must be re-implemented; it only needs verification and one commit so all later tasks start from a clean tree.

**Files:**
- Commit: the 33 already-modified tracked files and 3 untracked modules
- Do **not** stage: `.freebuff/` (handled in Task 12)

**Interfaces:**
- Consumes: existing working tree (already builds and passes tests)
- Produces: a clean `main` with the sixth pass landed, so Tasks 1–12 diff against a committed baseline

- [ ] **Step 1: Verify the tree is green**

Run: `scripts/check.sh`
Expected: fmt clean, clippy `-D warnings` clean, all tests pass.

- [ ] **Step 2: Verify the TUI harness**

Run: `scripts/verify-tui.sh`
Expected: 39/39 checks pass (the script exits non-zero on any failure).

- [ ] **Step 3: Review the diff, then commit**

Run: `git diff --stat` and skim `git diff` — confirm only sixth-pass work is present. Then:

```bash
git add CHANGELOG.md docs/18-RELEASE-ENGINEERING.md docs/23-FEATURE-MATRIX.md docs/26-IMPLEMENTATION-STATUS.md scripts/verify-tui.sh tests/ crates/
git commit -m "feat(search): tiered ranking, symbols, architecture milestones, dependency features, graph"
```

Expected: commit contains no `.freebuff/` files. Verify with `git show --stat HEAD | grep freebuff` (empty output).

---

## Workstream A — Analysis intelligence depth

### Task 1: Language-aware complexity signal (docs/10 §2)

The hotspot/risk complexity input is currently a pure LOC proxy. docs/10 §2 lists `function count (heuristic)` as a complexity input and requires that a missing input be marked unavailable, never silently zeroed. The heuristic extractor (`gitx_analysis::symbols`) already counts functions; wire it into the pipeline's complexity signal and label the source.

**Files:**
- Modify: `crates/gitx-analysis/src/symbols.rs` (make `lang_of` public, add `function_count`)
- Modify: `crates/gitx-analysis/src/pipeline.rs` (complexity signal, `FileAnalysis` fields)
- Modify: `crates/gitx-cli/src/commands/analysis.rs` (hotspots/risk evidence lines)
- Test: `crates/gitx-analysis/src/symbols.rs` (unit), `crates/gitx-analysis/src/pipeline.rs` (unit), `tests/integration/pipeline.rs`

**Interfaces:**
- Consumes: `crate::symbols::extract_symbols(content: &str, lang: &str) -> Vec<Symbol>` (existing)
- Produces: `pub fn lang_of(path: &Path) -> Option<&'static str>` (made public); `pub fn function_count(content: &str, lang: &str) -> u32`; `FileAnalysis { …, fn_count: u32, complexity_source: &'static str }`; `fn complexity_raw(loc: u32, fn_count: u32) -> u64` (unit-testable, `loc + 30 * fn_count`)

- [ ] **Step 1: Write the failing unit tests**

In `crates/gitx-analysis/src/symbols.rs`, append to the existing `#[cfg(test)] mod tests`:

```rust
#[test]
fn function_count_counts_only_functions_and_methods() {
    let src = "pub struct S;\nimpl S {\n    pub fn method(&self) {}\n}\nfn helper() {}\nconst C: u32 = 1;\n";
    assert_eq!(function_count(src, "rust"), 2);
    assert_eq!(function_count(src, "python"), 0); // unknown lang → extractor finds nothing
}

#[test]
fn function_count_zero_for_unsupported_language() {
    assert_eq!(function_count("def f(): pass", "plaintext"), 0);
}
```

In `crates/gitx-analysis/src/pipeline.rs` `mod tests`, add:

```rust
#[test]
fn complexity_raw_weights_functions_over_lines() {
    // Docs/10 §2: a function is treated as roughly 30 lines of complexity.
    assert_eq!(complexity_raw(100, 0), 100);
    assert_eq!(complexity_raw(100, 3), 190);
    assert_eq!(complexity_raw(0, 0), 0);
}

#[test]
fn scale_u64_normalizes_by_max() {
    assert_eq!(scale_u64(0, 10), 0.0);
    assert_eq!(scale_u64(10, 10), 100.0);
    assert_eq!(scale_u64(5, 10), 50.0);
    assert_eq!(scale_u64(7, 0), 0.0); // empty corpus → 0, never NaN
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p gitx-analysis function_count complexity_raw scale_u64`
Expected: FAIL with "function not found" / "cannot find function".

- [ ] **Step 3: Implement**

In `crates/gitx-analysis/src/symbols.rs`:

```rust
/// Language for a path's extension (lowercase, no dot), or `None` for
/// unsupported languages. Public so the pipeline and CLI can label
/// complexity sources (docs/10 §2: never silently zero a missing input).
pub fn lang_of(path: &Path) -> Option<&'static str> { /* existing body */ }

/// Heuristic function/method count for `content` (docs/10 §2 complexity
/// signal). Returns `0` for languages without an extractor; callers keep
/// LOC as the always-available fallback and label the source.
pub fn function_count(content: &str, lang: &str) -> u32 {
    extract_symbols(content, lang)
        .into_iter()
        .filter(|s| matches!(s.kind.as_str(), "Function" | "Method"))
        .count() as u32
}
```

In `crates/gitx-analysis/src/pipeline.rs`:

```rust
/// Docs/10 §2 complexity signal: LOC plus a per-function weight (one
/// function ≈ 30 lines). Deterministic; LOC alone remains the fallback
/// for languages without an extractor.
fn complexity_raw(loc: u32, fn_count: u32) -> u64 {
    loc as u64 + 30 * fn_count as u64
}

fn scale_u64(value: u64, max: u64) -> f64 {
    if max == 0 {
        0.0
    } else {
        value as f64 / max as f64 * 100.0
    }
}
```

Replace pipeline step 5 (currently builds `locs: HashMap<PathBuf, u32>`) with a parallel pass collecting `(loc, fn_count)` — same `par_chunks` structure, but per path read the blob once, compute LOC, and when `crate::symbols::lang_of(path)` is `Some(lang)`, also `crate::symbols::function_count(&text, lang)`:

```rust
let signals: HashMap<PathBuf, (u32, u32)> = analysis_pool().install(|| {
    paths
        .par_chunks(chunk)
        .map(|chunk| {
            let local = Repository::open(&git_dir)?;
            let mut out = Vec::with_capacity(chunk.len());
            for path in chunk {
                let blob = local
                    .blob_at_path(head_commit.tree_id, path)
                    .ok()
                    .flatten();
                let loc = blob
                    .as_deref()
                    .map(|b| split_lines(b).len() as u32)
                    .unwrap_or(0);
                let lang = crate::symbols::lang_of(path);
                let fn_count = match (&blob, lang) {
                    (Some(b), Some(lang)) => {
                        let text = String::from_utf8_lossy(b);
                        crate::symbols::function_count(&text, lang)
                    }
                    _ => 0,
                };
                out.push((path.clone(), (loc, fn_count)));
            }
            Ok(out)
        })
        .collect::<anyhow::Result<Vec<Vec<(PathBuf, (u32, u32))>>>>()
        .map(|chunks| {
            chunks
                .into_iter()
                .flatten()
                .collect::<HashMap<PathBuf, (u32, u32)>>()
        })
})?;
```

In step 6, replace the complexity normalization:

```rust
let max_complexity = signals
    .values()
    .map(|(loc, fns)| complexity_raw(*loc, *fns))
    .max()
    .unwrap_or(0);
```

and inside the per-file loop:

```rust
let (loc, fn_count) = signals.get(path).copied().unwrap_or((0, 0));
let raw_complexity = complexity_raw(loc, fn_count);
let n_complexity = scale_u64(raw_complexity, max_complexity);
```

Add to `FileAnalysis` (pipeline.rs:23):

```rust
    /// Function/method count from the heuristic extractor (docs/10 §2).
    pub fn_count: u32,
    /// `"symbols+loc"` when the extractor ran, `"loc"` otherwise (docs/10 §2:
    /// never silently zero a missing input — label the fallback).
    pub complexity_source: &'static str,
```

and set both in `files.push(FileAnalysis { … })`:

```rust
            fn_count,
            complexity_source: if fn_count > 0 { "symbols+loc" } else { "loc" },
```

- [ ] **Step 4: Run the unit tests to verify they pass**

Run: `cargo test -p gitx-analysis`
Expected: PASS. If any pre-existing score assertion in `pipeline.rs` or `health.rs` tests breaks, update it to the new expected value (the fixture's Rust files now carry function-count complexity) — do not weaken the assertion.

- [ ] **Step 5: Label the evidence in the CLI**

In `crates/gitx-cli/src/commands/analysis.rs`, the hotspots and risk per-file evidence lines must name the source. Find the line printing per-file hotspot/risk evidence (contains the file path and `risk:`/`score:`) and append the complexity source:

```rust
// example shape; match the surrounding formatting style of the command
// (evidence-first, docs/25):
//   "complexity: symbols+loc (7 functions)" or "complexity: loc (unsupported language)"
```

Run: `cargo build -p gitx-cli && gitx hotspots --limit 3` (in the workspace repo — it is a real git repo) — verify the evidence line renders and no score is `0.0` for Rust files that contain functions.

- [ ] **Step 6: Regenerate and review snapshots**

Scores shift for fixture files, so the blessed snapshots change:

Run: `GITX_BLESS=1 cargo test --workspace --test cli_snapshots`
Expected: PASS. Then `git diff tests/snapshots/` and manually confirm each changed number is the *expected* direction (function-dense fixture files' hotspot/risk rose or stayed; nothing dropped to 0).

- [ ] **Step 7: Integration test — pipeline labels complexity honestly**

Append to `tests/integration/pipeline.rs`:

```rust
#[test]
fn complexity_source_is_labeled_per_language() {
    // Builds the standard edge-case fixture hermetically (same helper the
    // other pipeline tests use).
    let (_tmp, repo) = build_edge_fixture();
    let files = run_pipeline(&repo);
    let rust_file = files.iter().find(|f| f.path.ends_with(".rs")).expect("fixture has rust files");
    assert_eq!(rust_file.complexity_source, "symbols+loc");
    assert!(rust_file.fn_count >= 1);
    // A fixture file with no supported extension keeps the LOC fallback.
    let plain = files.iter().find(|f| f.path.extension().is_none()).unwrap_or_else(|| {
        files.iter().find(|f| f.fn_count == 0).expect("some file has no extractor")
    });
    assert_eq!(plain.complexity_source, "loc");
}
```

Run: `cargo test --workspace --test pipeline complexity_source`
Expected: PASS.

- [ ] **Step 8: Quality gates and commit**

Run: `scripts/check.sh`
Expected: green.
Commit:

```bash
git add crates/gitx-analysis/src/symbols.rs crates/gitx-analysis/src/pipeline.rs crates/gitx-cli/src/commands/analysis.rs tests/integration/pipeline.rs tests/snapshots/
git commit -m "feat(analysis): heuristic function-count complexity signal with labeled fallback"
```

---

### Task 2: Symbol history (`gitx symbols history`)

The `symbols` table and `gitx symbols` command only describe HEAD. docs/21 Stage 6 delivers *symbol history*: when a symbol appeared, moved, or vanished. Reuse the lineage engine (DRY): walk each file's mainline lineage, extract symbols per lineage commit, diff consecutive snapshots for the requested name. No schema change — this is live, deterministic analysis.

**Files:**
- Create: `crates/gitx-analysis/src/symbol_history.rs`
- Modify: `crates/gitx-analysis/src/lib.rs` (`pub mod symbol_history;`)
- Modify: `crates/gitx-cli/src/cli.rs` (Symbols subcommand gains an action)
- Modify: `crates/gitx-cli/src/commands/mod.rs` (dispatch)
- Modify: `crates/gitx-cli/src/commands/analysis.rs` (`symbol_history` command)
- Test: `tests/integration/symbol_history.rs` (new)

**Interfaces:**
- Consumes: `gitx_history::HistoryService::get_file_lineage(path: PathBuf, from: Option<ObjectId>) -> FileLineage` (existing); `crate::symbols::{extract_symbols, lang_of}` (Task 1); `gitx_git::Repository::{head_commit_id, find_commit, blob_at_path}` (existing)
- Produces: `pub struct SymbolEvent { pub commit_id: String, pub file: PathBuf, pub action: SymbolAction }`; `pub enum SymbolAction { Added { line: u32 }, Moved { from_line: u32, to_line: u32 }, Removed { line: u32 } }`; `pub fn symbol_history(repo: &Repository, name: &str, path: Option<&Path>) -> anyhow::Result<Vec<SymbolEvent>>`

- [ ] **Step 1: Write the failing integration test**

Create `tests/integration/symbol_history.rs` modeled on `tests/integration/lineage.rs` (which builds commits with `gix` in a temp dir):

```rust
use gitx_git::Repository;
use std::path::Path;

/// Build a temp repo: lib.rs created with `fn helper()` at line 1; commit B
/// prepends `use std::fmt;` (moves `helper` to line 3); commit C renames the
/// function to `helper2`.
fn build_symbol_fixture() -> (tempfile::TempDir, Repository) { /* same pattern as lineage.rs */ }

#[test]
fn symbol_history_tracks_add_move_remove() {
    let (_tmp, repo) = build_symbol_fixture();
    let events = gitx_analysis::symbol_history::symbol_history(&repo, "helper", None).unwrap();

    // Newest first (lineage order): helper2 was renamed, helper vanished.
    let removed = events.iter().find(|e| matches!(e.action, SymbolAction::Removed { .. })).expect("helper removed at C");
    let added = events.iter().rev().find(|e| matches!(e.action, SymbolAction::Added { line: 1 })).expect("helper added at A");
    assert!(events.iter().any(|e| matches!(e.action, SymbolAction::Moved { from_line: 1, to_line: 3 })));
}

#[test]
fn symbol_history_empty_for_unknown_symbol() {
    let (_tmp, repo) = build_symbol_fixture();
    let events = gitx_analysis::symbol_history::symbol_history(&repo, "no_such_fn", None).unwrap();
    assert!(events.is_empty());
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test --workspace --test symbol_history`
Expected: FAIL — module/function not found.

- [ ] **Step 3: Implement `symbol_history`**

Create `crates/gitx-analysis/src/symbol_history.rs`:

```rust
//! Symbol history (docs/21 Stage 6): when a named symbol appeared, moved,
//! or vanished along a file's mainline lineage. Deterministic and read-only;
//! reuses the lineage engine instead of a new schema.

use crate::symbols::{Symbol, extract_symbols, lang_of};
use gitx_git::Repository;
use gitx_history::HistoryService;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolAction {
    Added { line: u32 },
    Moved { from_line: u32, to_line: u32 },
    Removed { line: u32 },
}

#[derive(Debug, Clone)]
pub struct SymbolEvent {
    pub commit_id: String,
    pub file: PathBuf,
    pub action: SymbolAction,
}

/// Extract symbols for `path` at `tree_id`, sorted by line (deterministic).
fn symbols_at(repo: &Repository, tree_id: gitx_git::models::ObjectId, path: &Path) -> Vec<Symbol> {
    let Some(lang) = lang_of(path) else { return Vec::new(); };
    let Ok(Some(bytes)) = repo.blob_at_path(tree_id, path) else { return Vec::new(); };
    let content = String::from_utf8_lossy(&bytes);
    let mut symbols = extract_symbols(&content, lang);
    symbols.sort_by_key(|s| s.line);
    symbols
}

/// Walk the lineage of every HEAD file containing `name` and emit
/// Add/Move/Remove events, newest first (lineage order). `path` restricts
/// the search to a directory prefix.
pub fn symbol_history(
    repo: &Repository,
    name: &str,
    path: Option<&Path>,
) -> anyhow::Result<Vec<SymbolEvent>> {
    let head = repo.head_commit_id()?;
    let head_commit = repo.find_commit(head)?;
    let mut events = Vec::new();

    let files: Vec<PathBuf> = repo
        .list_blobs(head_commit.tree_id)?
        .into_iter()
        .filter(|p| path.map(|prefix| p.starts_with(prefix)).unwrap_or(true))
        .filter(|p| lang_of(p).is_some())
        .collect();

    let history = HistoryService::new(repo);
    for file in files {
        let lineage = history.get_file_lineage(file.clone(), None)?;
        // Snapshots are collected oldest→newest by walking the nodes in
        // reverse (nodes are newest first).
        let mut prev: Vec<Symbol> = Vec::new();
        let mut first = true;
        for node in lineage.history.iter().rev() {
            let commit = repo.find_commit(node.commit_id)?;
            let cur = symbols_at(repo, commit.tree_id, &node.path);
            if first {
                prev = cur;
                first = false;
                continue;
            }
            let prev_line = prev.iter().find(|s| s.name == name).map(|s| s.line);
            let cur_line = cur.iter().find(|s| s.name == name).map(|s| s.line);
            match (prev_line, cur_line) {
                (None, Some(line)) => events.push(SymbolEvent {
                    commit_id: node.commit_id.to_string(),
                    file: node.path.clone(),
                    action: SymbolAction::Added { line },
                }),
                (Some(_), None) => events.push(SymbolEvent {
                    commit_id: node.commit_id.to_string(),
                    file: node.path.clone(),
                    action: SymbolAction::Removed { line: prev_line.unwrap() },
                }),
                (Some(from), Some(to)) if from != to => events.push(SymbolEvent {
                    commit_id: node.commit_id.to_string(),
                    file: node.path.clone(),
                    action: SymbolAction::Moved { from_line: from, to_line: to },
                }),
                _ => {}
            }
            prev = cur;
        }
    }
    events.sort_by(|a, b| b.commit_id.cmp(&a.commit_id)); // newest first, stable
    Ok(events)
}
```

Add `pub mod symbol_history;` to `crates/gitx-analysis/src/lib.rs`.

- [ ] **Step 4: Wire the CLI subcommand**

In `crates/gitx-cli/src/cli.rs`, change `Symbols` to:

```rust
    /// Source symbols extracted from HEAD (heuristic; docs/21 Stage 6).
    Symbols {
        /// Restrict to a path prefix.
        #[arg(long)]
        path: Option<String>,
        #[command(subcommand)]
        action: Option<SymbolsAction>,
    },
```

and add (next to the other `#[derive(Subcommand)]` action enums):

```rust
#[derive(Subcommand, Debug, Clone)]
pub enum SymbolsAction {
    /// Life of a symbol: when it was added, moved, or removed along the
    /// mainline (docs/21 Stage 6).
    History { name: String },
}
```

In `crates/gitx-cli/src/commands/mod.rs`, change the dispatch to:

```rust
        Commands::Symbols { path, action } => match action {
            Some(crate::cli::SymbolsAction::History { name }) => {
                analysis::symbol_history(&cli, &name, path.as_deref())
            }
            None => analysis::symbols(&cli, path.as_deref()),
        },
```

- [ ] **Step 5: Implement the command output**

In `crates/gitx-cli/src/commands/analysis.rs`, add (human output is evidence-first, docs/25; JSON mirrors the existing `symbols` command):

```rust
pub fn symbol_history(cli: &Cli, name: &str, path: Option<&str>) -> anyhow::Result<()> {
    let repo = open_repo(cli)?;
    let events =
        gitx_analysis::symbol_history::symbol_history(&repo, name, path.map(std::path::Path::new))?;
    if cli.json {
        return print_json(&json!({
            "symbol": name,
            "events": events.iter().map(|e| json!({
                "commit": e.commit_id,
                "file": e.file.display().to_string(),
                "action": match &e.action {
                    gitx_analysis::symbol_history::SymbolAction::Added { line } => format!("added:{line}"),
                    gitx_analysis::symbol_history::SymbolAction::Moved { from_line, to_line } =>
                        format!("moved:{from_line}->{to_line}"),
                    gitx_analysis::symbol_history::SymbolAction::Removed { line } => format!("removed:{line}"),
                },
            })).collect::<Vec<_>>(),
        }));
    }
    if events.is_empty() {
        println!("No history for symbol `{name}` in HEAD files.");
        return Ok(());
    }
    println!("Symbol history: `{name}` (mainline, newest first)");
    for e in events {
        match &e.action {
            gitx_analysis::symbol_history::SymbolAction::Added { line } =>
                println!("  added   {}  {}:{line}", e.commit_id, e.file.display()),
            gitx_analysis::symbol_history::SymbolAction::Moved { from_line, to_line } =>
                println!("  moved   {}  {}:{from_line} -> :{to_line}", e.commit_id, e.file.display()),
            gitx_analysis::symbol_history::SymbolAction::Removed { line } =>
                println!("  removed {}  {}:{line}", e.commit_id, e.file.display()),
        }
    }
    Ok(())
}
```

- [ ] **Step 6: Run the tests**

Run: `cargo test --workspace --test symbol_history`
Expected: PASS.

- [ ] **Step 7: Manual smoke test**

Run: `cargo run -p gitx-cli -- symbols history lang_of` (in the workspace repo) — expect a `Moved`/`Added` history for the function, newest first. Then `cargo run -p gitx-cli -- symbols history no_such_symbol` — expect the empty message, exit 0.

- [ ] **Step 8: Quality gates and commit**

Run: `scripts/check.sh`
Commit:

```bash
git add crates/gitx-analysis/src/symbol_history.rs crates/gitx-analysis/src/lib.rs crates/gitx-cli/src/cli.rs crates/gitx-cli/src/commands/mod.rs crates/gitx-cli/src/commands/analysis.rs tests/integration/symbol_history.rs
git commit -m "feat(analysis): symbol history via lineage-based extraction diffing"
```

---

### Task 3: Structural graph depth (docs/02 V1 "stronger architecture graph")

`gitx graph` exists (file + directory nodes, Contains + Imports edges, JSON) but has no *call* edges, the `gitx_graph::dependency` module is dead code, and the TUI has no graph view. Add call edges from the heuristic symbol extractor, remove the dead module, and add a list-based TUI Graph view.

**Files:**
- Delete: `crates/gitx-graph/src/dependency.rs` (dead — manifest parsing is centralized in `gitx_analysis::manifest`)
- Modify: `crates/gitx-graph/src/lib.rs` (drop `pub mod dependency;`)
- Modify: `crates/gitx-cli/src/commands/analysis.rs` (`graph` gains call edges)
- Modify: `crates/gitx-tui/src/app.rs`, `crates/gitx-tui/src/ui.rs`, `crates/gitx-tui/src/lib.rs`, `crates/gitx-tui/src/views/architecture.rs` (new Graph view)
- Test: `tests/integration/graph.rs` (new); extend `scripts/verify-tui.sh`

**Interfaces:**
- Consumes: `gitx_graph::graph::CodeGraph { add_node, add_edge, get_node, graph: DiGraph }`, `EdgeType::Calls` (both exist); `crate::symbols::extract_symbols_from_tree` (exists)
- Produces: `gitx graph` JSON with `edge_type: "call"` and a `weight` equal to occurrence count (capped at 100); TUI view index 14 = Graph

- [ ] **Step 1: Write the failing integration test**

Create `tests/integration/graph.rs` building a 2-file Rust fixture (`lib.rs` calls `util::helper()`, `util.rs` defines it):

```rust
#[test]
fn graph_emits_import_and_call_edges() {
    let (_tmp, repo) = build_rust_fixture();
    let edges = gitx_graph_call_edges(&repo); // helper returning Vec<(String, String, String, u32)>
    assert!(edges.iter().any(|(a, b, t, w)| t == "import" && a.ends_with("lib.rs") && b.ends_with("util.rs")));
    assert!(edges.iter().any(|(a, b, t, w)| t == "call" && a.ends_with("lib.rs") && b.ends_with("util.rs") && *w >= 1));
    assert!(!edges.iter().any(|(_, _, t, _)| t == "call" && a == b), "no self-call edges");
}
```

(Expose the edge list by calling the same logic the CLI command uses; the cleanest seam is a small `pub fn build_head_code_graph(repo: &Repository) -> anyhow::Result<CodeGraph>` in `gitx-graph::graph` — move the tree/import/call scanning from `analysis::graph` into that function so CLI and tests share it.)

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test --workspace --test graph`
Expected: FAIL (no `build_head_code_graph`, no call edges).

- [ ] **Step 3: Implement shared graph builder + call edges**

Add to `crates/gitx-graph/src/graph.rs`:

```rust
use gitx_git::Repository;

/// Build the HEAD code graph: file + directory nodes, Contains edges,
/// heuristic Imports edges, and Call edges from the symbol extractor
/// (docs/02 V1, docs/21 Stage 6). Deterministic order; bounded.
pub fn build_head_code_graph(repo: &Repository) -> anyhow::Result<CodeGraph> {
    let head = repo.head_commit_id()?;
    let head_commit = repo.find_commit(head)?;
    let mut graph = CodeGraph::new();
    let mut dirs: std::collections::HashSet<String> = std::collections::HashSet::new();

    // Symbol index: function/method name -> owning file (first match wins,
    // iterating sorted paths, so it is deterministic).
    let symbols = gitx_analysis::symbols::extract_symbols_from_tree(repo, head_commit.tree_id)?;
    let mut owner: std::collections::HashMap<String, std::path::PathBuf> =
        std::collections::HashMap::new();
    for (path, syms) in &symbols {
        for s in syms {
            if matches!(s.kind.as_str(), "Function" | "Method") {
                owner.entry(s.name.clone()).or_insert_with(|| path.clone());
            }
        }
    }

    for path in repo.list_blobs(head_commit.tree_id)? {
        // ... existing node + directory + Contains + Imports logic from
        // analysis::graph (moved verbatim) ...
        // Then, for the same content, scan for calls: any `name(` where
        // `name` is a function owned by a *different* file.
        let text = String::from_utf8_lossy(&bytes);
        let mut calls: std::collections::BTreeMap<String, u32> = std::collections::BTreeMap::new();
        for (name, owner_path) in &owner {
            if *owner_path == path {
                continue; // no self edges
            }
            let count = text.matches(&format!("{name}(")).count() as u32;
            if count > 0 {
                *calls.entry(owner_path.display().to_string()).or_insert(0) += count.min(100);
            }
        }
        for (target, weight) in calls {
            if let (Some(a), Some(b)) = (graph.get_node(&path), graph.get_node(&target)) {
                graph.add_edge(a, b, EdgeType::Calls, weight);
            }
        }
    }
    Ok(graph)
}
```

In `crates/gitx-cli/src/commands/analysis.rs`, replace the body of `graph()` with a call to `gitx_graph::graph::build_head_code_graph(&repo)` and update the JSON edge emission to include `"type": "call" | "import" | "contains"` (match on `EdgeType`).

Delete `crates/gitx-graph/src/dependency.rs` and remove `pub mod dependency;` from `crates/gitx-graph/src/lib.rs`.

- [ ] **Step 4: Run the tests**

Run: `cargo test --workspace --test graph && cargo build --workspace`
Expected: PASS; workspace compiles without the deleted module.

- [ ] **Step 5: Add the TUI Graph view**

Add `View::Graph` to `crates/gitx-tui/src/app.rs` (enum at line 4, nav maps around line 345/480) at index 14, reusing the Architecture panel's module-table renderer: the Graph view shows a module table (directory → file count, import edge count, call edge count) computed from `build_head_code_graph` data loaded lazily (see Task 7) into a new `App` field `graph_summary: Option<Vec<(String, usize, usize, usize)>>`. Sidebar label: `Graph`. View-jump key: `g`.

Update `crates/gitx-tui/src/ui.rs` to render the table (copy the Architecture view's table render call and adjust columns) and `crates/gitx-tui/src/lib.rs` if it enumerates views.

- [ ] **Step 6: Extend the PTY harness**

Add 2 checks to `scripts/verify-tui.sh`: press `g` → frame contains the Graph table header (e.g. "directory"); press `g` then `Esc` → returns to sidebar navigation.

Run: `scripts/verify-tui.sh`
Expected: 41/41.

- [ ] **Step 7: Quality gates and commit**

Run: `scripts/check.sh`
Commit:

```bash
git add crates/gitx-graph/src/graph.rs crates/gitx-graph/src/lib.rs crates/gitx-cli/src/commands/analysis.rs crates/gitx-tui/src/app.rs crates/gitx-tui/src/ui.rs crates/gitx-tui/src/lib.rs scripts/verify-tui.sh tests/integration/graph.rs
git rm crates/gitx-graph/src/dependency.rs
git commit -m "feat(graph): call edges, shared head-graph builder, TUI graph view; drop dead dependency module"
```

---

### Task 4: Advanced filters (docs/02 V1)

Timeline and search are missing the remaining practical filters: committer, merge filtering, `search --until`, and path-scoped search.

**Files:**
- Modify: `crates/gitx-history/src/timeline.rs` (`TimelineOptions` + filter logic)
- Modify: `crates/gitx-cli/src/cli.rs` (timeline/search flags)
- Modify: `crates/gitx-cli/src/commands/history.rs` (timeline options wiring)
- Modify: `crates/gitx-services/src/search.rs` (`SearchOptions` + filter logic)
- Modify: `crates/gitx-cli/src/commands/search.rs` (wiring)
- Test: `crates/gitx-history/src/timeline.rs` (unit), `tests/integration/services.rs` (search)

**Interfaces:**
- Consumes: `TimelineOptions` (existing fields); `commit.committer: Signature { name, email }`; `commit.parents: Vec<ObjectId>` (both exist); `SearchOptions` (existing fields)
- Produces: `TimelineOptions { committer: Option<String>, merges_only: bool, no_merges: bool }`; `SearchOptions { until: Option<i64>, path: Option<String> }`

- [ ] **Step 1: Write the failing unit tests**

In `crates/gitx-history/src/timeline.rs` `mod tests` (add if none exists; otherwise create `tests/integration/history_blame.rs` coverage):

```rust
#[test]
fn timeline_filters_by_committer_and_merges() {
    // Build a temp repo with commit A (author Alice, committer Alice),
    // commit B (author Bob, committer Alice), merge commit M (2 parents).
    let (_tmp, repo) = build_committer_fixture();
    let svc = HistoryService::new(&repo);

    let all = svc.timeline(TimelineOptions::default()).unwrap();
    let by_committer = svc
        .timeline(TimelineOptions { committer: Some("alice@example.com".into()), ..Default::default() })
        .unwrap();
    assert_eq!(by_committer.len(), 2); // A + M
    assert!(by_committer.iter().all(|c| c.committer.email == "alice@example.com"));

    let merges = svc.timeline(TimelineOptions { merges_only: true, ..Default::default() }).unwrap();
    assert_eq!(merges.len(), 1);
    assert_eq!(merges[0].parents.len(), 2);

    let no_merges = svc.timeline(TimelineOptions { no_merges: true, ..Default::default() }).unwrap();
    assert!(no_merges.iter().all(|c| c.parents.len() <= 1));
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p gitx-history committer_filters_by`
Expected: FAIL (fields do not exist).

- [ ] **Step 3: Implement timeline filters**

In `crates/gitx-history/src/timeline.rs`, extend the struct and the filter block (currently at line 44):

```rust
pub struct TimelineOptions {
    // ...existing fields...
    /// Only commits whose committer name/email contains this.
    pub committer: Option<String>,
    /// Only merge commits (2+ parents).
    pub merges_only: bool,
    /// Exclude merge commits.
    pub no_merges: bool,
}
```

In `timeline()`, next to the existing author filter:

```rust
            if let Some(committer) = &options.committer {
                let name_matches = commit.committer.name.contains(committer);
                let email_matches = commit.committer.email.contains(committer);
                if !name_matches && !email_matches {
                    continue;
                }
            }
            if options.merges_only && commit.parents.len() < 2 {
                continue;
            }
            if options.no_merges && commit.parents.len() >= 2 {
                continue;
            }
```

- [ ] **Step 4: Wire the timeline CLI flags**

In `crates/gitx-cli/src/cli.rs` `Timeline { … }`:

```rust
        /// Only commits whose committer name/email contains this.
        #[arg(long)]
        committer: Option<String>,
        /// Only merge commits.
        #[arg(long)]
        merges: bool,
        /// Exclude merge commits.
        #[arg(long)]
        no_merges: bool,
```

In `crates/gitx-cli/src/commands/history.rs`, populate `TimelineOptions` from the flags.

- [ ] **Step 5: Search `--until` and `--path`**

In `crates/gitx-services/src/search.rs`:

```rust
pub struct SearchOptions {
    // ...existing fields...
    /// Only commits at or before this unix timestamp.
    pub until: Option<i64>,
    /// Restrict file/symbol/directory scopes to this path prefix.
    pub path: Option<String>,
}
```

Apply in `search()`: the commit scope already filters `since`; add `AND c.author_time <= ?` (or equivalent post-filter on the fetched rows) for `until`. For file/symbol/directory scopes, append `AND f.path LIKE '<prefix>%'` to the SQL when `path` is set.

In `crates/gitx-cli/src/commands/search.rs`, parse `--until` with the same date parsing used for `--since` (docs/07 §19: RFC3339 or unix seconds) and `--path <PREFIX>` into `SearchOptions`.

- [ ] **Step 6: Tests for search filters**

Append to `tests/integration/services.rs`:

```rust
#[test]
fn search_until_and_path_scope_work() {
    let (_tmp, repo) = build_edge_fixture();
    let svc = SearchService::new(&repo);
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
    let hits = svc.search("*", "*", &SearchOptions {
        files: true,
        until: Some(now - 86_400 * 365), // a year ago → nothing in a fresh fixture
        ..Default::default()
    }).unwrap();
    assert!(hits.is_empty(), "until filter must exclude fresh commits");
    let hits = svc.search("*", "*", &SearchOptions { files: true, ..Default::default() }).unwrap();
    assert!(!hits.is_empty());
    let scoped = svc.search("*", "*", &SearchOptions { files: true, path: Some("nonexistent/".into()), ..Default::default() }).unwrap();
    assert!(scoped.is_empty());
}
```

Run: `cargo test --workspace --test services search_until`
Expected: PASS.

- [ ] **Step 7: Quality gates and commit**

Run: `scripts/check.sh`
Commit:

```bash
git add crates/gitx-history/src/timeline.rs crates/gitx-cli/src/cli.rs crates/gitx-cli/src/commands/history.rs crates/gitx-services/src/search.rs crates/gitx-cli/src/commands/search.rs tests/integration/services.rs tests/integration/history_blame.rs
git commit -m "feat(cli): committer, merge, and search until/path filters"
```

---

## Workstream B — CLI breadth

### Task 5: `gitx diff` with streamed, bounded output (docs/13 §4/§8)

No `gitx diff` exists. docs/13 §8 requires large diffs to be paginated/streamed rather than materialized whole. Implement tree-to-tree diff with per-file streaming into a pager.

**Files:**
- Modify: `crates/gitx-git/src/diff.rs` (per-file patch renderer)
- Modify: `crates/gitx-cli/src/cli.rs` (`Diff` command)
- Modify: `crates/gitx-cli/src/commands/mod.rs` (dispatch + `Commands::Diff` arm)
- Create: `crates/gitx-cli/src/commands/diff.rs`
- Test: `tests/integration/diff.rs` (new)

**Interfaces:**
- Consumes: `Repository::diff_tree_to_tree(old_tree: Option<ObjectId>, new_tree: ObjectId) -> Result<Vec<FileChange>>`; `Repository::{find_commit, head_commit_id, blob_at_path, peel_ref?}` — resolve refs with `repo.find_commit(repo.resolve_ref(name)?)` (confirm `resolve_ref` exists in `gitx-git`; if not, reuse the same resolution `release diff` uses); `crate::commands::paginate(Vec<String>)`
- Produces: `gitx-git::diff::render_file_patch(repo, old_tree: Option<ObjectId>, new_tree: ObjectId, change: &FileChange) -> Result<Option<String>>`; CLI `gitx diff <FROM> <TO> [PATH] [--stat]` with `less -R` streaming on a TTY

- [ ] **Step 1: Write the failing integration test**

Create `tests/integration/diff.rs`:

```rust
#[test]
fn diff_stat_matches_change_counts() {
    let (_tmp, repo) = build_edge_fixture();
    // Use the existing fixture builder: from = first commit, to = HEAD.
    let head = repo.head_commit_id().unwrap();
    let first = /* oldest commit on mainline */;
    let changes = repo.diff_tree_to_tree(Some(repo.find_commit(first).unwrap().tree_id), repo.find_commit(head).unwrap().tree_id).unwrap();
    assert!(!changes.is_empty());

    for change in &changes {
        let patch = gitx_git::diff::render_file_patch(&repo, Some(repo.find_commit(first).unwrap().tree_id), repo.find_commit(head).unwrap().tree_id, change).unwrap();
        let patch = patch.expect("changed file has a patch");
        let plus = patch.lines().filter(|l| l.starts_with('+') && !l.starts_with("+++")).count();
        let minus = patch.lines().filter(|l| l.starts_with('-') && !l.starts_with("---")).count();
        assert_eq!(plus as u32, change.insertions, "insertions match for {}", change.path.display());
        assert_eq!(minus as u32, change.deletions, "deletions match for {}", change.path.display());
    }
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test --workspace --test diff`
Expected: FAIL (function not found).

- [ ] **Step 3: Implement the per-file patch renderer**

In `crates/gitx-git/src/diff.rs`, factor the per-file unified-diff body out of `render_commit_patch` (lines 260–336) into:

```rust
/// Render a unified patch for one file change between two trees (docs/13
/// §8: callers stream files one at a time so only one file's hunks are in
/// memory). Returns `None` when both blobs are missing.
pub fn render_file_patch(
    repo: &Repository,
    old_tree: Option<ObjectId>,
    new_tree: ObjectId,
    change: &FileChange,
) -> Result<Option<String>> {
    let old_path = change.old_path.as_ref().unwrap_or(&change.path);
    let old_bytes = repo.blob_at_path(old_tree.unwrap_or(new_tree), old_path).ok().flatten();
    let new_bytes = repo.blob_at_path(new_tree, &change.path).ok().flatten();
    if old_bytes.is_none() && new_bytes.is_none() {
        return Ok(None);
    }
    // ...exact body from render_commit_patch's per-change section
    // (diff --git header, mode lines, ---/+++, @@ hunk)...
    Ok(Some(out))
}
```

and make `render_commit_patch` call it in its loop (one call site, behavior unchanged — the existing recovery tests lock this).

- [ ] **Step 4: Add the CLI command**

In `crates/gitx-cli/src/cli.rs`:

```rust
    /// Unified diff between two refs, streamed per file (docs/13 §8).
    Diff {
        from: String,
        to: String,
        /// Restrict to a path prefix.
        path: Option<String>,
        /// Summary only (file list + insertions/deletions).
        #[arg(long)]
        stat: bool,
    },
```

Create `crates/gitx-cli/src/commands/diff.rs`:

```rust
use crate::Cli;
use crate::commands::paginate;

pub fn diff(cli: &Cli, from: &str, to: &str, path: Option<&str>, stat: bool) -> anyhow::Result<()> {
    let repo = open_repo(cli)?;
    let from_id = resolve_ref(&repo, from)?; // same ref-resolution used by release diff
    let to_id = resolve_ref(&repo, to)?;
    let from_commit = repo.find_commit(from_id)?;
    let to_commit = repo.find_commit(to_id)?;
    let changes = repo.diff_tree_to_tree(Some(from_commit.tree_id), to_commit.tree_id)?;

    let mut lines: Vec<String> = Vec::new();
    let header = format!("diff {} -> {} ({} files)", from, to, changes.len());
    lines.push(header);

    if stat {
        for change in changes.iter().filter(|c| path.map(|p| c.path.starts_with(p)).unwrap_or(true)) {
            let mark = match change.change_type {
                gitx_git::models::ChangeType::Added => "A",
                gitx_git::models::ChangeType::Deleted => "D",
                gitx_git::models::ChangeType::Renamed => "R",
                gitx_git::models::ChangeType::Copied => "C",
                gitx_git::models::ChangeType::Modified => "M",
            };
            lines.push(format!(
                "{mark}  {:+} / {:-}  {}",
                change.insertions, change.deletions, change.path.display()
            ));
        }
        return paginate(lines);
    }

    // Stream per file: render one patch at a time (memory bounded, docs/13
    // §8) and write it into the pager's stdin on a TTY.
    let mut child = if std::io::IsTerminal::is_terminal(&std::io::stdout()) {
        Some(std::process::Command::new("less")
            .arg("-R")
            .stdin(std::process::Stdio::piped())
            .spawn()?)
    } else {
        None
    };
    let mut sink: Box<dyn std::io::Write> = match &mut child {
        Some(c) => Box::new(c.stdin.take().unwrap()),
        None => Box::new(std::io::stdout()),
    };
    for change in changes.iter().filter(|c| path.map(|p| c.path.starts_with(p)).unwrap_or(true)) {
        if let Some(patch) =
            gitx_git::diff::render_file_patch(&repo, Some(from_commit.tree_id), to_commit.tree_id, change)?
        {
            writeln!(sink, "{patch}")?;
        }
    }
    drop(sink);
    if let Some(mut c) = child {
        c.wait()?;
    }
    Ok(())
}
```

Add the dispatch arm in `crates/gitx-cli/src/commands/mod.rs` and `pub mod diff;`.

- [ ] **Step 5: Run the tests and smoke-test the memory behavior**

Run: `cargo test --workspace --test diff`
Expected: PASS.

Smoke: `cargo run -p gitx-cli -- diff <TAG1> <TAG2> --stat` and `… | head -20` on the workspace repo — verify headers, counts, and that piping works without a pager.

- [ ] **Step 6: Document the command**

Add `gitx diff` to `docs/07-CLI-SPECIFICATION.md` (a new §17.5 or next to release diff) with the streaming/pagination note and the `--stat` flag; note the memory bound in `docs/13 §8` is now satisfied (mark item 59's diff half closed in `docs/26`).

- [ ] **Step 7: Quality gates and commit**

Run: `scripts/check.sh`
Commit:

```bash
git add crates/gitx-git/src/diff.rs crates/gitx-cli/src/cli.rs crates/gitx-cli/src/commands/mod.rs crates/gitx-cli/src/commands/diff.rs tests/integration/diff.rs docs/07-CLI-SPECIFICATION.md docs/13-PERFORMANCE-ENGINEERING.md docs/26-IMPLEMENTATION-STATUS.md
git commit -m "feat(cli): streamed gitx diff with stat mode and paged output"
```

---

### Task 6: CSV export (docs/02 V2 "richer export formats")

Analytical commands print human text and `--json`; add a deterministic CSV output for the tabular commands (hotspots, contributors, risk, health, ownership, branches, timeline) with a hand-rolled writer (no new dependency).

**Files:**
- Create: `crates/gitx-core/src/csv.rs`
- Modify: `crates/gitx-core/src/lib.rs` (`pub mod csv;`)
- Modify: `crates/gitx-cli/src/cli.rs` (global `--csv` flag)
- Modify: `crates/gitx-cli/src/commands/mod.rs` (shared `emit_table` helper)
- Modify: tabular command bodies in `crates/gitx-cli/src/commands/analysis.rs`, `commands/history.rs` (route through `emit_table` when `--csv`)
- Test: `crates/gitx-core/src/csv.rs` (unit), `tests/integration/cli_snapshots.rs` (CLI CSV)

**Interfaces:**
- Consumes: `Cli { json: bool, csv: bool }` (json exists)
- Produces: `gitx_core::csv::write_csv(headers: &[String], rows: &[Vec<String>]) -> String` (RFC-4180-ish: quote fields containing `,` `"` `\n`; double inner quotes; CRLF line endings); `fn emit_table<T: Serialize + ToRow>(cli: &Cli, headers, rows)` in `commands/mod.rs`

- [ ] **Step 1: Write the failing unit tests**

In `crates/gitx-core/src/csv.rs`:

```rust
#[test]
fn write_csv_quotes_and_escapes() {
    let out = write_csv(
        &["name".to_string(), "score".to_string()],
        &vec![
            vec!["a,b".to_string(), "1".to_string()],
            vec!["say \"hi\"".to_string(), "2".to_string()],
            vec!["line1\nline2".to_string(), "3".to_string()],
        ],
    );
    assert_eq!(
        out,
        "name,score\r\n\"a,b\",1\r\n\"say \"\"hi\"\"\",2\r\n\"line1\nline2\",3\r\n"
    );
}

#[test]
fn write_csv_plain_fields_unquoted() {
    let out = write_csv(&["a".to_string()], &vec![vec!["plain".to_string()]]);
    assert_eq!(out, "a\r\nplain\r\n");
}
```

- [ ] **Step 2: Run to verify they fail**

Run: `cargo test -p gitx-core write_csv`
Expected: FAIL (module/function not found).

- [ ] **Step 3: Implement the writer**

```rust
//! Minimal deterministic CSV writer (docs/02 V2 richer export formats).
//! Hand-rolled to avoid a new dependency; RFC-4180 quoting.

/// Serialize `rows` (parallel to `headers`) as CSV. Fields containing a
/// comma, double quote, or newline are quoted; embedded quotes are doubled.
pub fn write_csv(headers: &[String], rows: &[Vec<String>]) -> String {
    let mut out = String::new();
    out.push_str(&escape_row(headers));
    for row in rows {
        out.push_str(&escape_row(row));
    }
    out
}

fn escape_row(fields: &[String]) -> String {
    let mut line = String::new();
    for (i, field) in fields.iter().enumerate() {
        if i > 0 {
            line.push(',');
        }
        if field.contains(',') || field.contains('"') || field.contains('\n') {
            line.push('"');
            line.push_str(&field.replace('"', "\"\""));
            line.push('"');
        } else {
            line.push_str(field);
        }
    }
    line.push_str("\r\n");
    line
}
```

Add `pub mod csv;` to `crates/gitx-core/src/lib.rs`.

- [ ] **Step 4: Run the unit tests**

Run: `cargo test -p gitx-core write_csv`
Expected: PASS.

- [ ] **Step 5: Wire the CLI**

In `crates/gitx-cli/src/cli.rs` add a global flag next to `--json`:

```rust
    /// CSV output (tabular analytical commands only; overrides --json).
    #[arg(long, global = true)]
    csv: bool,
```

In `crates/gitx-cli/src/commands/mod.rs` add the shared helper:

```rust
/// Route tabular output through human / JSON / CSV (docs/02 V2). Returns
/// true when a non-human format was emitted.
pub fn emit_table<T>(
    cli: &Cli,
    title: &str,
    headers: &[String],
    rows: &[Vec<String>],
    to_rows: impl Fn(&T) -> Vec<String>,
) -> anyhow::Result<()> {
    if cli.csv {
        print!("{}", gitx_core::csv::write_csv(headers, rows));
        return Ok(());
    }
    // existing human rendering with `title`; keep current behavior otherwise
    crate::commands::render_human_table(cli, title, headers, rows);
    Ok(())
}
```

(If the commands currently print rows inline rather than through a table helper, refactor *one* command — `hotspots` — to collect `Vec<Vec<String>>` rows first, then call `emit_table`; keep JSON path untouched. The other commands follow the identical pattern; do them one at a time, re-running the CLI snapshot test after each so drift is caught immediately.)

- [ ] **Step 6: Verify via snapshots and smoke test**

Run: `cargo test --workspace --test cli_snapshots`
Expected: PASS (human output unchanged).

Smoke: `cargo run -p gitx-cli -- hotspots --csv --limit 3` and `… | python3 -c "import csv,sys; print(list(csv.reader(sys.stdin)))"` — verify parseable CSV, headers first.

- [ ] **Step 7: Quality gates and commit**

Run: `scripts/check.sh`
Commit:

```bash
git add crates/gitx-core/src/csv.rs crates/gitx-core/src/lib.rs crates/gitx-cli/src/cli.rs crates/gitx-cli/src/commands/mod.rs crates/gitx-cli/src/commands/analysis.rs crates/gitx-cli/src/commands/history.rs tests/integration/cli_snapshots.rs
git commit -m "feat(cli): deterministic CSV export for tabular commands"
```

---

## Workstream C — TUI and performance

### Task 7: Lazy panel loading for sub-second startup (docs/13 §3/§7)

`load_repo_stats` computes every panel's data in one eager pass (stats, timeline, hotspots, contributors, dependencies, recovery, arch diff, branch intel). docs/13 §3 targets sub-second usability with a fresh index and §7 mandates lazy loading. Split the load into two phases: Phase A (Overview essentials: stats, activity, timeline) lands first and renders immediately; Phase B (hotspots, contributors, dependencies, recovery, arch diff, branch intel) loads on a second background pass while the user navigates, with the existing stage/step progress bar.

**Files:**
- Modify: `crates/gitx-tui/src/app.rs` (`LoadMsg` phases, `spawn_load`/`reload`, `apply_data`, per-panel `Option` gates, view render guard)
- Modify: `crates/gitx-tui/src/views/*.rs` (render "Loading…" empty state until the panel's data is present — reuse `common::empty_rows`)
- Modify: `crates/gitx-tui/src/lib.rs` (`load_repo_stats` split)
- Modify: `scripts/verify-tui.sh` (startup-timing + lazy-panel checks)

**Interfaces:**
- Consumes: `App::apply_data(&mut self, data: AppData)` (existing); `LoadMsg` channel (existing); `common::empty_rows` (existing)
- Produces: `enum LoadMsg { Phase { data: AppData }, Done }` — `apply_data` applies a partial `AppData` and `loading` stays true until `Done`; views render placeholders for `None` panels

- [ ] **Step 1: Add the failing PTY check**

In `scripts/verify-tui.sh`, after the existing startup checks add:

```bash
# Lazy loading: the Overview must render before heavy panels land.
expect_frame_startup  # starts gitx-tui, captures the first frame after 0.3s
grep -q "Commits" "$FRAME" || fail "overview does not render at startup"
grep -q "Analyzing" "$STATUS" || fail "status bar shows a load stage"
# Recovery panel must show a loading placeholder before its data arrives.
send_keys "r"  # jump to Recovery (no 'v' key collision; use the sidebar)
expect_frame  # frame may still be loading
grep -q "Loading" "$FRAME" || pass_hint "recovery data already present"
```

(Adjust the keys to the harness's existing helpers; the goal is: startup frame shows Overview immediately, and a heavy panel visited early shows a loading state rather than stale/empty data.)

- [ ] **Step 2: Run to verify it fails**

Run: `scripts/verify-tui.sh`
Expected: FAIL (heavy panels currently render as empty/blank before data lands, or the check greps a missing marker).

- [ ] **Step 3: Split the load**

In `crates/gitx-tui/src/lib.rs` `load_repo_stats`, compute Phase A first (stats, activity, timeline, timeline_file_counts, repo_state — the Overview essentials), send `LoadMsg::Phase { data }`, then compute Phase B (hotspots, contributors, dependencies, recovery, arch_diff, branch_intel, health_evidence, commit_files) and send `LoadMsg::Phase { data }` followed by `LoadMsg::Done`. Keep the existing per-stage progress messages and Esc-cancel semantics (check the cancel flag between phases).

In `crates/gitx-tui/src/app.rs`:

- `enum LoadMsg { Phase { data: AppData }, Done }`
- `apply_data` handles `Phase` (merge fields; `loading` unchanged) and `Done` (`loading = false`)
- the tick loop consumes `LoadMsg` variants accordingly
- each heavy view's render call sites guard on `self.<panel>.is_some()`, falling back to `common::empty_rows("Loading… (background load in progress)")`

- [ ] **Step 4: Run the harness**

Run: `scripts/verify-tui.sh`
Expected: 40/40+ (previous 39 + the new startup/lazy checks). Fix any frame-grep mismatches; do not weaken assertions.

- [ ] **Step 5: Manual timing check**

Run: `time scripts/verify-tui.sh` and compare Overview-appears latency before/after (note the numbers in the task log). With a fresh index, the Overview should paint within ~1s on the dev machine (docs/13 §3 budget).

- [ ] **Step 6: Quality gates and commit**

Run: `scripts/check.sh`
Commit:

```bash
git add crates/gitx-tui/src/app.rs crates/gitx-tui/src/lib.rs crates/gitx-tui/src/views/ scripts/verify-tui.sh
git commit -m "perf(tui): phased lazy panel loading for faster startup"
```

---

### Task 8: Benchmark breadth and a regression gate (docs/13 §9/§10)

Benches exist for analysis (health) and services (index build + search). docs/13 §9 asks for medium/large/merge-heavy/rename-heavy repositories and benchmarks for file history, branch analysis, and search; §10 asks for stored results and a regression policy.

**Files:**
- Modify: `crates/gitx-services/benches/operations.rs` (fixture generators + new benches)
- Modify: `crates/gitx-analysis/benches/health.rs` (merge-heavy/rename-heavy fixture notes)
- Create: `scripts/bench.sh`
- Create: `benches/RESULTS.md` (recorded baseline)
- Modify: `docs/13-PERFORMANCE-ENGINEERING.md` (§9/§10 pointers)

**Interfaces:**
- Consumes: existing hermetic fixture builder in `crates/gitx-services/benches/operations.rs`; `HistoryService::get_file_lineage`; `SearchService::search`; `branch_intelligence`
- Produces: `scripts/bench.sh` (runs `cargo bench --workspace`, appends a timestamped section to `benches/RESULTS.md`); baseline numbers recorded

- [ ] **Step 1: Write the failing benchmark (compilation is the test)**

Extend `crates/gitx-services/benches/operations.rs` with three fixture generators (deterministic, in-memory temp repos built with gix):

```rust
fn medium_repo(c: &mut Criterion) { /* ~2k commits, 300 files */ }
fn merge_heavy_repo(c: &mut Criterion) { /* ~1k commits with periodic 2-parent merges */ }
fn rename_heavy_repo(c: &mut Criterion) { /* ~1k commits renaming files periodically */ }
```

and three benches:

```rust
fn bench_file_lineage(c: &mut Criterion) { /* get_file_lineage on a deep-history file */ }
fn bench_branch_analysis(c: &mut Criterion) { /* branch_intelligence over all branches */ }
fn bench_fts_search(c: &mut Criterion) { /* SearchService::search with a realistic query */ }
```

Run: `cargo bench -p gitx-services --no-run`
Expected: FAIL until the bench harness compiles (write them iteratively until `--no-run` passes; the "test" here is that the benchmark harness itself compiles and runs on the small fixture).

- [ ] **Step 2: Run the benches and record baselines**

Run: `cargo bench -p gitx-services -- --warm-up-time 1 --measurement-time 3` (dev-machine run; keep CI compile-only per the current workflow).
Expected: each bench reports a mean and p-value distribution.

Create `benches/RESULTS.md`:

```markdown
# Benchmark results

Run with `scripts/bench.sh`. Format: crate / bench / mean / date / host.

| Crate | Bench | Mean | Date | Host |
|---|---|---|---|---|
| gitx-services | index build (medium) | <fill from output> | 2026-08-12 | <host> |
| ... | | | | |
```

- [ ] **Step 3: Create the runner script**

`scripts/bench.sh`:

```bash
#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")/.."
DATE="$(date +%Y-%m-%d)"
HOST="$(uname -sm)"
cargo bench --workspace -- --warm-up-time 1 --measurement-time 3 > /tmp/gitx-bench.log
echo "" >> benches/RESULTS.md
echo "## $DATE ($HOST)" >> benches/RESULTS.md
grep -E "time:|change:" /tmp/gitx-bench.log | head -40 >> benches/RESULTS.md
```

`chmod +x scripts/bench.sh` and run it once to populate the baseline.

- [ ] **Step 4: Document the regression policy**

In `docs/13-PERFORMANCE-ENGINEERING.md` §10, replace the prose with a concrete gate: "A >20% regression vs the recorded baseline in `benches/RESULTS.md` for index build, search, or file history must be investigated before release; re-run `scripts/bench.sh` and compare." Keep it honest about variance (warm-up, host differences).

- [ ] **Step 5: Commit**

Run: `scripts/check.sh` (bench compile included in CI via `cargo bench --no-run`), then:

```bash
git add crates/gitx-services/benches/operations.rs crates/gitx-analysis/benches/health.rs scripts/bench.sh benches/RESULTS.md docs/13-PERFORMANCE-ENGINEERING.md
git commit -m "bench: medium/merge/rename fixtures, file-history/branch/search benches, recorded baseline"
```

---

## Workstream D — Docs, release, hygiene

### Task 9: Tree-sitter decision — ADR-011 and the dead parser stub (docs/02 V1, docs/20, docs/23)

ADR-011 is "Proposed" but the heuristic symbol extractor already satisfies the docs/23 feature rows; `gitx-graph::treesitter` is a no-op `DummyParser` that nothing consumes. Decide and record: defer Tree-sitter with rationale (consistent with the feature-freeze principle and the deterministic-only philosophy), remove the misleading stub, and lock the behavior with a test.

**Files:**
- Delete: `crates/gitx-graph/src/treesitter.rs`; remove `pub mod treesitter;` from `crates/gitx-graph/src/lib.rs`
- Modify: `docs/20-ADR.md` (ADR-011 status → Accepted: deferred, with the trigger criteria)
- Modify: `docs/23-FEATURE-MATRIX.md` (note: no parser stub remains; heuristic extractor is the implementation)
- Test: `crates/gitx-analysis/src/symbols.rs` (default-behavior lock)

**Interfaces:**
- Consumes: nothing (pure removal + docs)
- Produces: documented decision; green workspace without `treesitter.rs`

- [ ] **Step 1: Write the behavior-lock test**

In `crates/gitx-analysis/src/symbols.rs`:

```rust
#[test]
fn default_symbol_extraction_is_heuristic() {
    // Locks the default: no Tree-sitter adapter is wired (ADR-011 deferred);
    // the deterministic line-based extractor is the implementation.
    let src = "pub fn a() {}\nfn b() {}\n";
    let syms = extract_symbols(src, "rust");
    assert_eq!(syms.len(), 2);
    assert!(syms.iter().all(|s| matches!(s.kind.as_str(), "Function")));
}
```

- [ ] **Step 2: Run to verify it passes (the stub is the thing being removed)**

Run: `cargo test -p gitx-analysis default_symbol_extraction_is_heuristic`
Expected: PASS already — this test *documents* the decision; it starts failing only if someone wires a different default later.

- [ ] **Step 3: Remove the stub and update the decision**

```bash
git rm crates/gitx-graph/src/treesitter.rs
```

Edit `crates/gitx-graph/src/lib.rs` to drop `pub mod treesitter;`.

In `docs/20-ADR.md`, change ADR-011:

```markdown
### Status

Accepted (deferred)

### Decision

Use Tree-sitter adapters for language-aware analysis when the base
repository/file model is mature.

### Status note (2026-08-12)

Deferred. The deterministic line-based extractor (`gitx_analysis::symbols`)
already covers function/method/struct/class/enum/const symbols across 10+
languages and feeds search, symbol history, and the complexity signal. A
Tree-sitter adapter would add a heavy native dependency for marginal gain
against the feature-freeze principle. Revisit when: (a) call-graph accuracy
needs AST-level resolution beyond the heuristic `name(` scan, or (b) a
specific language's extractor proves unreliable in practice. The former
`gitx-graph::treesitter` placeholder was removed because nothing consumed it.
```

In `docs/23-FEATURE-MATRIX.md`, update the Tree-sitter note: the optional parser placeholder no longer exists; heuristic symbols are the implementation and Tree-sitter remains deferred per ADR-011.

- [ ] **Step 4: Verify**

Run: `scripts/check.sh`
Expected: green; `grep -rn "DummyParser\|treesitter" crates/` returns nothing.

- [ ] **Step 5: Commit**

```bash
git add crates/gitx-graph/src/lib.rs crates/gitx-analysis/src/symbols.rs docs/20-ADR.md docs/23-FEATURE-MATRIX.md
git commit -m "docs: defer tree-sitter (ADR-011), remove unconsumed parser stub"
```

---

### Task 10: Package-manager installation docs (docs/18 §9)

docs/18 §9 still says package-manager distribution is "planned but not yet published". cargo-dist is configured (`[workspace.metadata.dist]` in the root `Cargo.toml`), so the installers it generates are the story. Document the real install paths.

**Files:**
- Modify: `docs/18-RELEASE-ENGINEERING.md` (§9 "Package-manager installation")
- Modify: `docs/27-RELEASING.md` (checklist note)
- Test: docs build (markdown lint if configured; otherwise a manual read-through) + `cargo dist plan` if cargo-dist is installed

- [ ] **Step 1: Verify the cargo-dist surface**

Run: `cargo dist plan` (if cargo-dist is installed) — capture the generated installer commands (the `cargo dist` install script URL, the Homebrew tap name derived from the repository, and the per-target archives).

Expected: output lists `gitx` and `gitx-tui`, the four targets, and installer recipes. If cargo-dist is not installed, use the documented cargo-dist 0.24.0 conventions with the repo's actual metadata (owner is unknown in the config — use `USER` placeholder as the existing docs do).

- [ ] **Step 2: Rewrite the "later" section**

Replace the placeholder in `docs/18-RELEASE-ENGINEERING.md`:

```markdown
### Package-manager installation

Tagged releases publish cargo-dist installers alongside the archives:

- **curl installer (macOS/Linux):**
  `curl -LsSf https://github.com/USER/gitx/releases/latest/download/gitx-installer.sh | sh`
- **PowerShell installer (Windows):**
  `irm https://github.com/USER/gitx/releases/latest/download/gitx-installer.ps1 | iex`
- **Homebrew:** cargo-dist publishes a tap at
  `github.com/USER/homebrew-gitx`; install with
  `brew install USER/gitx/gitx`
- **Cargo (source):** `cargo install --path crates/gitx-cli --locked` and
  `cargo install --path crates/gitx-tui --locked`

Verify any install with `gitx --version` and `gitx-tui --version`. The
installers are generated by cargo-dist from `[workspace.metadata.dist]`;
regenerate the plan with `cargo dist plan` before a release (docs/27 §3).
```

- [ ] **Step 3: Update the release checklist**

In `docs/27-RELEASING.md`, add a checklist item: "Run `cargo dist plan`; confirm installers (shell/powershell/homebrew) are listed; copy the resulting install commands into docs/18 §9 if the tap/URL shape changed."

- [ ] **Step 4: Verify and commit**

Run: `grep -rn "planned but not yet published" docs/` — expected empty.
Commit:

```bash
git add docs/18-RELEASE-ENGINEERING.md docs/27-RELEASING.md
git commit -m "docs: document cargo-dist installers and homebrew tap"
```

---

### Task 11: Advanced copy/rename lineage (docs/02 V2)

Lineage records `Copied` changes nowhere (they fall through the match), copy detection only runs at file birth (root commit or re-Add), and no node records whether it arrived via a merge. Close the reliable, deterministic parts of docs/02 V2.

**Files:**
- Modify: `crates/gitx-history/src/lineage.rs`
- Test: `tests/integration/lineage.rs`

**Interfaces:**
- Consumes: `ChangeType::Copied` (exists in `gitx-git::models`); `FileLineageNode { commit_id, path, action }`
- Produces: `FileAction::Added { copy_of: Option<PathBuf> }` now emitted for `Copied` changes (source = `old_path`); `FileLineageNode { commit_id, path, action, via_merge: bool }`

- [ ] **Step 1: Write the failing integration tests**

Append to `tests/integration/lineage.rs`:

```rust
#[test]
fn copied_change_reports_copy_source() {
    // Fixture: util.rs exists; commit adds copy.rs with byte-identical
    // content to util.rs (gix diff reports ChangeType::Copied).
    let (_tmp, repo) = build_copy_fixture();
    let lineage = HistoryService::new(&repo).get_file_lineage(PathBuf::from("copy.rs"), None).unwrap();
    let added = lineage.history.iter().find(|n| matches!(n.action, FileAction::Added { .. }))
        .expect("copy.rs has an Added node");
    match &added.action {
        FileAction::Added { copy_of } => assert_eq!(copy_of.as_deref(), Some(Path::new("util.rs"))),
        other => panic!("expected Added with copy source, got {other:?}"),
    }
}

#[test]
fn merge_touch_is_marked_via_merge() {
    let (_tmp, repo) = build_merge_fixture();
    let lineage = HistoryService::new(&repo).get_file_lineage(PathBuf::from("shared.txt"), None).unwrap();
    assert!(lineage.history.iter().any(|n| n.via_merge), "merge commit touching the file is marked");
}
```

- [ ] **Step 2: Run to verify they fail**

Run: `cargo test --workspace --test lineage copied_change merge_touch`
Expected: FAIL (`via_merge` field missing; copied arm absent).

- [ ] **Step 3: Implement**

In `crates/gitx-history/src/lineage.rs`:

```rust
pub struct FileLineageNode {
    pub commit_id: ObjectId,
    pub path: PathBuf,
    pub action: FileAction,
    /// True when the change came in via a merge commit (2+ parents).
    pub via_merge: bool,
}
```

In `get_file_lineage`, compute once per commit: `let via_merge = commit.parents.len() > 1;` and set it on every node pushed in that iteration.

Add the `Copied` arm to the match (after the `Renamed` arm):

```rust
                    gitx_git::models::ChangeType::Copied if change.path == current => {
                        history.push(FileLineageNode {
                            commit_id,
                            path: current.clone(),
                            action: FileAction::Added {
                                copy_of: change.old_path.clone(),
                            },
                            via_merge,
                        });
                    }
```

Note: a `Copied` change does not rewind `current` (the copied file has its own independent life), so it must not set `renamed = true`.

- [ ] **Step 4: Run the tests**

Run: `cargo test --workspace --test lineage`
Expected: PASS (existing lineage tests still green — the new field defaults to `false` for non-merge commits).

- [ ] **Step 5: Surface the marker in the CLI**

In `crates/gitx-cli/src/commands/history.rs` (the `lineage` command), suffix merge-touched rows with `(merge)` when `node.via_merge` — match the existing row format.

- [ ] **Step 6: Quality gates and commit**

Run: `scripts/check.sh`
Commit:

```bash
git add crates/gitx-history/src/lineage.rs crates/gitx-cli/src/commands/history.rs tests/integration/lineage.rs
git commit -m "feat(history): copy-source lineage for Copied changes, merge-touch markers"
```

---

### Task 12: Repository hygiene (`.freebuff`, scratch files, `Cargo.lock`)

The Freebuff app database (`.freebuff/*.db*`, ~8 MB, churning on every session) is tracked; `test_gix.rs` and `fix_repo.sh` are committed scratch files; and `Cargo.lock` is gitignored even though `cargo install --locked` and cargo-dist reproducibility depend on it.

**Files:**
- Modify: `.gitignore`
- Delete (tracking): `.freebuff/desktop-v2.db*`, `test_gix.rs`, `fix_repo.sh`
- Add: `Cargo.lock`
- Modify: `docs/27-RELEASING.md` (note)

**Interfaces:**
- Consumes: nothing
- Produces: clean tracked tree (no local-tool data, no scratch); reproducible `--locked` builds

- [ ] **Step 1: Update `.gitignore`**

Append to `.gitignore`:

```gitignore
# Freebuff app-local data (never commit)
.freebuff/
```

and remove the `Cargo.lock` line so the lockfile is committed (binary/application workspace convention; needed for `--locked` installs documented in docs/18 §9).

- [ ] **Step 2: Untrack and delete**

```bash
git rm --cached .freebuff/desktop-v2.db .freebuff/desktop-v2.db-shm .freebuff/desktop-v2.db-wal
git rm test_gix.rs fix_repo.sh
git add Cargo.lock
```

The `.freebuff/` files stay on disk (local tool data); only tracking is removed.

- [ ] **Step 3: Verify**

Run: `git status --porcelain | grep freebuff` — expected empty. Run: `cargo build --locked --workspace` — expected success (proves the committed lockfile is usable).

- [ ] **Step 4: Document**

In `docs/27-RELEASING.md`, add under the versioning section: "`Cargo.lock` is committed (binary workspace); bump it with `cargo update` deliberately, and always verify `cargo build --locked` in the release checklist."

- [ ] **Step 5: Commit**

```bash
git add .gitignore docs/27-RELEASING.md Cargo.lock
git commit -m "chore: untrack freebuff app data and scratch files, commit Cargo.lock"
```

---

## Final pass

- [ ] **Task 13: Whole-workspace verification**

Run `scripts/check.sh` and `scripts/verify-tui.sh`, then update `docs/26-IMPLEMENTATION-STATUS.md` with a "Seventh implementation pass" section summarizing each task above (mirroring the existing pass sections), and refresh the `docs/23-FEATURE-MATRIX.md` notes for any rows touched (symbol history, graph call edges, CSV export, `gitx diff`). Update `CHANGELOG.md` under `[Unreleased]`. Commit:

```bash
git add docs/26-IMPLEMENTATION-STATUS.md docs/23-FEATURE-MATRIX.md CHANGELOG.md
git commit -m "docs: seventh implementation pass status"
```

---

## Self-review

**Spec coverage:** Every open item from the audit table maps to exactly one task (0–13). docs/02 V1 "advanced filters" → Task 4; "richer metrics" → Task 1; "stronger architecture graph" → Task 3; docs/02 V2 "advanced copy/rename lineage" → Task 11; "richer export formats" → Task 6; "large-repository performance tuning" → Tasks 5, 7, 8; docs/21 Stage 6 "symbol history" → Task 2; docs/13 §9/§10 → Task 8; docs/18 §9 → Task 10; ADR-011 → Task 9. The sixth pass landing is Task 0, hygiene is Task 12.

**Placeholder scan:** Every code step contains concrete code or a concrete command. The two deliberately prose-level edits (Task 1 Step 5 evidence line, Task 6 Step 5 `emit_table` wiring) point to the exact function/line and the exact string to add; both are covered by an integration or snapshot test that fails if the edit is skipped.

**Type consistency:** Names used across tasks match existing code: `symbols::extract_symbols(content: &str, lang: &str) -> Vec<Symbol>`, `HistoryService::get_file_lineage(path: PathBuf, from: Option<ObjectId>) -> FileLineage`, `diff_tree_to_tree(Option<ObjectId>, ObjectId) -> Vec<FileChange>`, `FileChange { path, old_path, change_type, insertions, deletions }`, `EdgeType::{Contains, Imports, Calls}`, `TimelineOptions`/`SearchOptions` field names, `paginate(Vec<String>)`. Task 1's `lang_of` rename is the only cross-file signature change and is flagged at the interface line. Task 11's `via_merge` field is added in the same task that reads it; Task 3's `build_head_code_graph` is introduced in the same task that consumes it.
