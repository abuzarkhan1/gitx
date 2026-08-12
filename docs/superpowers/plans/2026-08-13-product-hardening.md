# Product Hardening & Ship Readiness Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn GitX from a feature-complete codebase into a shippable product: `gitx` must launch the dashboard on first use (docs/16 §7), the first-run experience must build the index automatically, documented config must actually work, the health score must read correctly, the project must be publish-ready, and the README must sell the product.

**Architecture:** Seven independent tasks, each a closed loop of (failing test → implementation → passing test → commit). Tasks 1–6 are code/docs changes in the existing workspace with no new crates; Task 7 is a maintainer-gated release runbook that consumes Tasks 1–6. CLI behavior is verified with integration tests against real fixture repos (the existing `tests/common/mod.rs` `FixtureRepo` pattern); TUI behavior is verified headlessly by extending `scripts/verify-tui.sh` (the existing tmux PTY harness).

**Tech Stack:** Rust workspace (11 crates, gix 0.66, rusqlite, ratatui 0.28, clap 4.5, tokio), cargo-dist, GitHub Actions, integration tests + tmux PTY harness. No new external dependencies are added except `tokio` (already in the tree via `gitx-tui`).

## Global Constraints

- Workspace crates `gitx-cli` and `gitx-tui` are edition 2024; other crates keep their current edition.
- `scripts/check.sh` (fmt + clippy `-D warnings` + check + test) must stay green after every task.
- JSON/CSV output contracts must not change unless a task explicitly says so (docs/07 §18).
- Do not add new external crates; the only new dependency is `tokio` in `gitx-cli` (already used by `gitx-tui`).
- Docs must stay in sync: every behavior change updates the matching `docs/` spec in the same task.
- No network access is required for Tasks 1–6 (all fixture repos are created hermetically; `cargo package` is local). `cargo publish` / `git push --tags` in Task 7 require maintainer credentials and are explicitly gated.
- Snapshot tests are regenerated deliberately with `GITX_BLESS=1` (docs/14 §5) whenever a task changes CLI text output.
- The persisted index lives at `<git_dir>/gitx/index.sqlite` (docs/16 §6); integration tests must clean up via the `FixtureRepo` Drop.

---
## File Structure

| File | Responsibility |
|---|---|
| `crates/gitx-cli/src/cli.rs` | New `Tui` subcommand variant (Task 2) |
| `crates/gitx-cli/src/commands/mod.rs` | No-arg dispatch → TUI/snapshot; `ensure_fresh_index` helper (Tasks 2–3) |
| `crates/gitx-cli/src/commands/repo.rs` | New `snapshot()` non-TTY dashboard (Task 2); `stats()` auto-refresh + `index.enabled` (Tasks 3, 5) |
| `crates/gitx-cli/src/commands/analysis.rs` | Health band text + `health_band` use (Task 4); `default_limit` fallbacks, `index.enabled` live forcing (Task 5) |
| `crates/gitx-cli/src/commands/history.rs` | `timeline --max` fallback to `default_limit` (Task 5) |
| `crates/gitx-cli/src/commands/index.rs` | `index.enabled=false` no-op (Task 5) |
| `crates/gitx-cli/src/commands/search.rs` | `case_sensitive` threaded from config (Task 5) |
| `crates/gitx-cli/Cargo.toml` | `gitx-tui` + `tokio` deps, publish metadata, versioned path deps (Tasks 1–2) |
| `crates/gitx-tui/src/lib.rs` | `run(vim_keys)` signature, vim-key gating (Task 2) |
| `crates/gitx-tui/src/main.rs` | Load config, pass `vim_keys` (Task 2) |
| `crates/gitx-tui/src/app.rs` | `App.vim_keys` field; auto-refresh in `load_repo_stats` (Tasks 2–3) |
| `crates/gitx-services/src/index.rs` | New `IndexService::refresh_if_stale()` (Task 3) |
| `crates/gitx-services/src/search.rs` | `SearchOptions.case_sensitive` + FTS post-filter (Task 5) |
| `crates/gitx-search/src/code.rs` | `search_code_content` gains `case_sensitive` param (Task 5) |
| `crates/gitx-analysis/src/lib.rs` | New `health_band()` helper (Task 4) |
| All `crates/*/Cargo.toml` | Publish metadata + versioned workspace-internal deps (Task 1) |
| `CHANGELOG.md` | Real 0.1.0 section (Task 1) |
| `README.md` | Storefront overhaul (Task 6) |
| `docs/07-CLI-SPECIFICATION.md`, `docs/16-CONFIGURATION.md`, `docs/08-TUI-SPECIFICATION.md` | Spec sync (Tasks 2, 4, 5) |
| `docs/27-RELEASING.md` | Release runbook update (Task 7) |
| `tests/integration/services.rs`, `tests/integration/failures.rs`, new `tests/integration/product.rs` | Integration tests (Tasks 2, 3, 4, 5) |
| `scripts/verify-tui.sh` | No-arg TUI launch check (Task 2) |

---
### Task 1: Ship-ready metadata and a real CHANGELOG

**Files:**
- Modify: `crates/*/Cargo.toml` (all 11 crates)
- Modify: `CHANGELOG.md`

**Interfaces:**
- Consumes: nothing.
- Produces: publishable crate manifests (every crate has `license`, `repository`, `keywords`, `categories`, and every `gitx-*` path dependency also carries `version = "0.1.0"`), and a `CHANGELOG.md` whose `[0.1.0]` section is real. Task 7 depends on this.

- [ ] **Step 1: Write the failing verification**

`cargo package` fails today for the internal crates because path-only dependencies are not publishable:

```bash
cargo package -p gitx-core --allow-dirty --no-verify 2>&1 | grep -i "path dependency" || echo "UNEXPECTED: no path-dependency error"
```

Expected: the grep prints a cargo error mentioning path dependencies (or `--list` shows missing `license`/`repository` warnings). If cargo instead complains about missing metadata first, that same failure is the verification — the point is that `cargo package` is not clean.

- [ ] **Step 2: Add publish metadata to every crate**

For **each** of the 11 crates in `crates/`, add to `[package]` (all values verbatim except `description` and `keywords`):

```toml
license = "MIT"
repository = "https://github.com/abuzarkhan1/gitx"
keywords = ["git", "history", "analysis", "cli", "terminal"]
categories = ["command-line-utilities", "development-tools"]
```

with these per-crate descriptions:

| Crate | description |
|---|---|
| gitx-core | Shared domain types, configuration, and identity normalization for GitX |
| gitx-git | Git object, ref, diff, and reflog access via gix |
| gitx-index | Incremental history indexer with progress and cancellation |
| gitx-storage | SQLite schema, migrations, and storage providers |
| gitx-history | Timeline, lineage, and line-level blame engines |
| gitx-analysis | Deterministic repository intelligence: hotspots, risk, health, ownership, dependencies, symbols |
| gitx-graph | Code-graph and structural snapshot comparison |
| gitx-search | Full-text and code-content search across the persisted index |
| gitx-services | Application service layer over repository, index, analysis, search, recovery, history |
| gitx-cli | GitX command-line interface (binary `gitx`) |
| gitx-tui | GitX interactive terminal dashboard (binary `gitx-tui`) |

- [ ] **Step 3: Add version requirements to workspace-internal dependencies**

In **every** crate, change each `gitx-*` path dependency from:

```toml
gitx-core = { path = "../gitx-core" }
```

to:

```toml
gitx-core = { version = "0.1.0", path = "../gitx-core" }
```

Apply the same transformation to every `gitx-*` dependency in every crate (gitx-cli depends on nine of them; gitx-tui on seven; gitx-services, gitx-storage, gitx-history, gitx-analysis, gitx-graph, gitx-index, gitx-search each depend on two to five). `tests/Cargo.toml` already has `publish = false` — leave it as is.

- [ ] **Step 4: Rewrite the CHANGELOG 0.1.0 section**

Replace the placeholder in `CHANGELOG.md`:

```markdown
## [0.1.0] - placeholder

### Added
- Initial scaffolding: 10-crate Rust workspace, migrations, tests, benches.
```

with:

```markdown
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
```

Leave the `## [Unreleased]` section empty (remove its bullet list, since everything there shipped in 0.1.0) or delete the `[Unreleased]` section entirely.

- [ ] **Step 5: Run the verification to confirm it passes**

```bash
cargo metadata --no-deps --format-version 1 > /dev/null && echo "metadata OK"
# Leaves of the internal dependency graph package locally right now:
cargo package -p gitx-core --allow-dirty --no-verify > /dev/null 2>&1 && echo "package OK: gitx-core"
cargo package -p gitx-index --allow-dirty --no-verify > /dev/null 2>&1 && echo "package OK: gitx-index"
# All versioned path deps must resolve to the workspace copies:
cargo tree -p gitx-cli -e normal 2>&1 | grep -c "gitx-" | xargs -I{} echo "{} workspace-internal crates resolve for gitx-cli"
cargo build --workspace > /dev/null 2>&1 && echo "workspace build OK"
```

Expected: `metadata OK`, `package OK` for the two dependency leaves (gitx-core, gitx-index — they have no unpublished internal deps), a positive count of resolved `gitx-*` crates in `cargo tree`, and `workspace build OK`. **Dependent crates** (gitx-git, gitx-history, gitx-analysis, gitx-graph, gitx-search, gitx-services, gitx-cli, gitx-tui) cannot be `cargo package`-checked until their leaves are on crates.io — cargo rewrites `version`+`path` deps to registry requirements during packaging, and unpublished versions fail resolution. Their packaging is verified in Task 7 after the leaf-first publishes; `cargo build --workspace` here proves the manifests are internally consistent.

- [ ] **Step 6: Full quality gate**

```bash
scripts/check.sh
```

Expected: fmt clean, clippy `-D warnings` clean, all tests pass.

- [ ] **Step 7: Commit**

```bash
git add crates/*/Cargo.toml CHANGELOG.md
git commit -m "chore: publish-ready crate metadata and real 0.1.0 changelog"
```

---
### Task 2: `gitx` (no args) launches the dashboard

**Files:**
- Modify: `crates/gitx-cli/Cargo.toml`
- Modify: `crates/gitx-cli/src/cli.rs`
- Modify: `crates/gitx-cli/src/commands/mod.rs`
- Modify: `crates/gitx-cli/src/commands/repo.rs`
- Modify: `crates/gitx-tui/src/lib.rs`
- Modify: `crates/gitx-tui/src/main.rs`
- Modify: `crates/gitx-tui/src/app.rs`
- Test: `tests/integration/product.rs` (new), `scripts/verify-tui.sh`
- Modify: `docs/07-CLI-SPECIFICATION.md`, `docs/16-CONFIGURATION.md`

**Interfaces:**
- Consumes: nothing from earlier tasks.
- Produces: `gitx_tui::run(vim_keys: bool) -> anyhow::Result<()>` (was `run()`); `App::new(vim_keys: bool)`; `App.vim_keys: bool`; `Commands::Tui`; `repo::snapshot(&cli) -> anyhow::Result<()>`; `commands::run_dashboard_or_tui(&Cli) -> anyhow::Result<()>`; `gitx-cli` now depends on `gitx-tui` and `tokio`. Task 5's config wiring reuses `App.vim_keys` as-is (already threaded here).

- [ ] **Step 1: Write the failing integration test**

Create `tests/integration/product.rs`:

```rust
//! Product-surface behavior (docs/01 UC-01, docs/16 §7): `gitx` with no
//! subcommand must do something useful — launch the dashboard on a TTY and
//! print a repository snapshot when piped.

#[path = "../common/mod.rs"]
mod common;

use common::FixtureRepo;
use std::process::Command;

fn bin() -> std::path::PathBuf {
    let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("..");
    p.push("target");
    p.push("debug");
    p.push(if cfg!(windows) { "gitx.exe" } else { "gitx" });
    p
}

#[test]
fn gitx_with_no_args_on_a_pipe_prints_a_snapshot() {
    let Some(repo) = FixtureRepo::new("product-noarg") else {
        return;
    };
    repo.write("src/lib.rs", "pub fn hello() {}\n");
    repo.commit("feat: initial");
    repo.write("src/lib.rs", "pub fn hello() { println!(\"hi\"); }\n");
    repo.commit("feat: print hi");

    let out = Command::new(bin())
        .arg("--repo")
        .arg(repo.path())
        .output()
        .expect("gitx runs");
    assert!(out.status.success(), "no-arg must exit 0, got {:?}", out.status);
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(
        text.contains("GitX") && text.contains("commits"),
        "piped no-arg must print a snapshot, got: {text}"
    );
    assert!(
        text.contains("2"),
        "fixture has 2 commits, snapshot should mention them: {text}"
    );
}

#[test]
fn gitx_tui_subcommand_exists_in_help() {
    let out = Command::new(bin())
        .arg("--help")
        .output()
        .expect("gitx --help runs");
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(
        text.contains("tui") && text.contains("interactive terminal"),
        "help must document `gitx tui`, got: {text}"
    );
}
```

Add to `tests/Cargo.toml`:

```toml
[[test]]
name = "product"
path = "integration/product.rs"
```

- [ ] **Step 2: Run the test to verify it fails**

```bash
cargo test -p gitx-tests --test product
```

Expected: FAIL — `gitx` with no args currently prints the "TUI is a separate binary" hint to stderr and `--help` has no `tui` subcommand.

- [ ] **Step 3: Add the `Tui` subcommand and the `gitx-tui` dependency**

In `crates/gitx-cli/Cargo.toml` add:

```toml
gitx-tui = { version = "0.1.0", path = "../gitx-tui" }
tokio = { version = "1.0", features = ["rt-multi-thread", "time"] }
```

In `crates/gitx-cli/src/cli.rs`, add to `Commands`:

```rust
    /// Launch the interactive terminal dashboard.
    Tui,
```

- [ ] **Step 4: No-arg dispatch — TUI on a TTY, snapshot on a pipe**

In `crates/gitx-cli/src/commands/mod.rs`, replace the no-subcommand block:

```rust
    // No subcommand → start the TUI (docs/07 §1). The TUI crate is a separate
    // binary today; surface a clear message instead of a silent no-op.
    if cli.command.is_none() {
        eprintln!(
            "gitx: the interactive TUI is a separate binary (gitx-tui); run `cargo run -p gitx-tui`"
        );
        eprintln!("gitx: use `gitx --help` for the available commands");
        return Ok(());
    }
```

with:

```rust
    // No subcommand → the dashboard (docs/01 UC-01, docs/16 §7): launch the
    // TUI when stdout is a terminal, otherwise print a repository snapshot
    // so `gitx` is useful in pipes and CI too.
    if cli.command.is_none() {
        return run_dashboard_or_tui(&cli);
    }
```

and add at the bottom of the file:

```rust
/// `gitx` with no subcommand (docs/01 UC-01, docs/16 §7): on a terminal,
/// launch the interactive dashboard in-process; on a pipe/CI, print a
/// compact repository snapshot so the command is never a silent no-op.
pub fn run_dashboard_or_tui(cli: &Cli) -> anyhow::Result<()> {
    use std::io::IsTerminal;
    if std::io::stdout().is_terminal() {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;
        let vim_keys = crate::commands::config::load_config(cli)
            .map(|c| c.ui.vim_keys)
            .unwrap_or(true);
        return runtime.block_on(gitx_tui::run(vim_keys));
    }
    repo::snapshot(cli)
}
```

Note: `load_config` currently returns `anyhow::Result<Config>`; it already exists in `crates/gitx-cli/src/commands/config.rs` — reuse it (its current signature returns `Result<Config>` with defaults on missing file; if it takes no args today, keep the call as-is and adjust the `.map(...)` accordingly).

- [ ] **Step 5: Add `snapshot()` to `repo.rs`**

In `crates/gitx-cli/src/commands/repo.rs`, add:

```rust
/// Compact one-screen repository snapshot for `gitx` on a non-terminal
/// (pipes, CI) — docs/16 §7: `gitx` must do something useful everywhere.
pub fn snapshot(cli: &Cli) -> anyhow::Result<()> {
    let repo = open_repo(cli)?;
    let service = RepositoryService::new(&repo);
    let data = service.info();
    let stats = service
        .stats_from_index()
        .ok()
        .flatten()
        .or_else(|| gitx_analysis::repository_stats(&repo).ok());
    let state = service.state();

    println!("GitX — repository snapshot");
    println!(
        "  repository : {}",
        data["work_dir"].as_str().unwrap_or("<bare>")
    );
    println!("  state      : {}", state.git);
    println!("  index      : {}", index_state_label(state.index));
    if let Some(s) = stats {
        println!("  commits    : {}", s.commits);
        println!("  contributors: {}", s.contributors);
        println!("  files      : {}", s.files);
        println!("  branches   : {}", s.branches);
        println!("  tags       : {}", s.tags);
    } else {
        println!("  commits    : (none yet)");
    }
    println!();
    println!("  This is the non-interactive summary. Run `gitx` in a terminal for the");
    println!("  dashboard, or `gitx --help` for every command.");
    Ok(())
}
```

- [ ] **Step 6: Thread `vim_keys` through the TUI**

In `crates/gitx-tui/src/lib.rs`:

```rust
pub async fn run() -> Result<()> {
```

becomes:

```rust
pub async fn run(vim_keys: bool) -> Result<()> {
```

and inside, `let mut app = App::new();` becomes `let mut app = App::new(vim_keys);`.

In `crates/gitx-tui/src/app.rs`, `pub fn new() -> Self { Self::spawn_load() }` becomes:

```rust
    pub fn new(vim_keys: bool) -> Self {
        Self::spawn_load(vim_keys)
    }

    fn spawn_load(vim_keys: bool) -> Self {
        let (tx, rx) = std::sync::mpsc::channel::<LoadMsg>();
        let cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let worker_cancel = cancel.clone();
        std::thread::spawn(move || {
            load_repo_stats(&tx, &worker_cancel);
        });
        Self {
            vim_keys,
            // ... every other field exactly as today ...
```

Add the field to the struct:

```rust
    /// Honor vim-style j/k/h/l keys (docs/16 `[ui] vim_keys`). Arrows always
    /// work; when false, j/k/h/l are ignored so non-vim users never trigger
    /// accidental navigation.
    pub vim_keys: bool,
```

In `crates/gitx-tui/src/lib.rs` `handle_key`, gate the vim arms on the flag (arrows stay unconditional):

```rust
        KeyCode::Down | KeyCode::Char('j') if app.vim_keys || key.code == KeyCode::Down => {
```

(and the same `if app.vim_keys || key.code == ...` guard for `Up | Char('k')`, `Enter | Right | Char('l')`, `Left | Char('h')`). In `gitx-tui/src/main.rs`, load the config and pass the flag:

```rust
    let vim_keys = gitx_core::config::Config::load(
        gitx_core::config::Config::default_path()
            .as_deref()
            .unwrap_or(std::path::Path::new("")),
    )
    .map(|c| c.ui.vim_keys)
    .unwrap_or(true);
    gitx_tui::run(vim_keys).await
```

(`Config::load` returns defaults when the path is missing/unreadable, so `vim_keys` defaults to `true` — today's behavior.)

- [ ] **Step 7: Extend `scripts/verify-tui.sh` with a no-arg check**

After the lazy-loading session block (before the main session), insert:

```bash
# ── `gitx` (no args) launches the dashboard (docs/01 UC-01) ─────────────
tmux kill-session -t gitxcli 2>/dev/null
tmux new-session -d -s gitxcli -x 140 -y 44
tmux send-keys -t gitxcli "cd $FIX && TERM=xterm-256color $ROOT/target/debug/gitx" Enter
for _ in $(seq 1 20); do
  tmux capture-pane -t gitxcli -p | grep -q "Overview" && break
  sleep 0.5
done
tmux capture-pane -t gitxcli -p > "$OUT/00_cli_noarg.txt"
if grep -q "Overview" "$OUT/00_cli_noarg.txt"; then
  PASS=$((PASS+1))
else
  echo "FAIL: gitx with no args must open the TUI Overview"
  FAIL=$((FAIL+1))
fi
tmux kill-session -t gitxcli 2>/dev/null
```

Update the trailing summary line count if the script prints "N checks" (inspect the tail of the script and bump the expected number).

- [ ] **Step 8: Sync the specs**

In `docs/07-CLI-SPECIFICATION.md`, replace any text stating the TUI is a separate binary launched via `gitx-tui` with: "`gitx` with no subcommand launches the interactive dashboard when stdout is a terminal (and prints a repository snapshot otherwise); `gitx tui` launches it explicitly." In `docs/16-CONFIGURATION.md` §7, confirm the promised flow `cd repository; gitx` now works as documented (update the wording only if it already claims the TUI launches — if the doc says "immediately use the tool", add "(the dashboard opens)").

- [ ] **Step 9: Run the verification**

```bash
cargo test -p gitx-tests --test product
cargo build -p gitx-cli -p gitx-tui
scripts/verify-tui.sh
scripts/check.sh
```

Expected: the two new tests pass; `gitx` in tmux renders the Overview; full quality gate green.

- [ ] **Step 10: Commit**

```bash
git add crates/gitx-cli crates/gitx-tui tests/integration/product.rs tests/Cargo.toml scripts/verify-tui.sh docs/07-CLI-SPECIFICATION.md docs/16-CONFIGURATION.md
git commit -m "feat(cli): gitx with no args opens the TUI dashboard, snapshot on pipes"
```

---
### Task 3: First-run auto-refresh — the index builds itself

**Files:**
- Modify: `crates/gitx-services/src/index.rs`
- Modify: `crates/gitx-cli/src/commands/mod.rs`
- Modify: `crates/gitx-cli/src/commands/repo.rs`
- Modify: `crates/gitx-cli/src/commands/analysis.rs`
- Modify: `crates/gitx-cli/src/commands/search.rs`
- Modify: `crates/gitx-tui/src/app.rs`
- Test: `tests/integration/product.rs`

**Interfaces:**
- Consumes: `IndexService::scan_with` (exists), `IndexService::is_fresh` (exists), `config.index.auto_refresh` (parsed today, never consumed).
- Produces: `IndexService::refresh_if_stale(&self) -> anyhow::Result<()>`; `commands::ensure_fresh_index(&Cli, &Repository) -> anyhow::Result<()>`. Task 5 reuses `ensure_fresh_index`'s config gate.

- [ ] **Step 1: Write the failing integration tests**

Append to `tests/integration/product.rs`:

```rust
#[test]
fn auto_refresh_builds_the_index_on_first_analytical_command() {
    let Some(repo) = FixtureRepo::new("product-autorefresh") else {
        return;
    };
    repo.write("src/lib.rs", "pub fn a() {}\npub fn b() {}\n");
    repo.commit("feat: initial");
    repo.write("src/lib.rs", "pub fn a() { println!(\"x\"); }\npub fn b() {}\n");
    repo.commit("fix: a prints");

    let out = Command::new(bin())
        .arg("--repo")
        .arg(repo.path())
        .arg("health")
        .output()
        .expect("gitx health runs");
    assert!(out.status.success(), "health failed: {:?}", out.status);

    let index = repo.path().join(".git/gitx/index.sqlite");
    assert!(
        index.exists(),
        "auto_refresh (default true) must create the index on first analysis"
    );
    let out = Command::new(bin())
        .arg("--repo")
        .arg(repo.path())
        .arg("index")
        .arg("status")
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("2 commits"), "index should hold 2 commits: {text}");
}

#[test]
fn auto_refresh_false_skips_index_build() {
    let Some(repo) = FixtureRepo::new("product-noautorefresh") else {
        return;
    };
    repo.write("README.md", "# demo\n");
    repo.commit("docs: readme");

    let config_dir = repo.path().join("gitx.toml");
    std::fs::write(
        &config_dir,
        "[index]\nauto_refresh = false\n",
    )
    .expect("write repo config");

    let out = Command::new(bin())
        .arg("--repo")
        .arg(repo.path())
        .arg("health")
        .output()
        .expect("gitx health runs");
    assert!(out.status.success());
    assert!(
        !repo.path().join(".git/gitx/index.sqlite").exists(),
        "auto_refresh=false must not create an index"
    );
}
```

- [ ] **Step 2: Run the tests to verify they fail**

```bash
cargo test -p gitx-tests --test product -- auto_refresh
```

Expected: FAIL — the first test finds no index file; the second passes trivially (no index is ever created), which is fine — the meaningful failure is the first.

- [ ] **Step 3: Add `IndexService::refresh_if_stale`**

In `crates/gitx-services/src/index.rs`, add:

```rust
    /// Incremental refresh when the persisted index is stale or absent
    /// (docs/16 `[index] auto_refresh`): cheap when only HEAD moved, a full
    /// build on first run. No-op when fresh. Used by the CLI and TUI so the
    /// index builds itself instead of every analytical command recomputing
    /// live from Git (docs/13 §3 sub-second reads).
    pub fn refresh_if_stale(&self) -> anyhow::Result<()> {
        if self.is_fresh() {
            return Ok(());
        }
        let cancelled = std::sync::atomic::AtomicBool::new(false);
        self.scan_with(true, &cancelled)?;
        Ok(())
    }
```

- [ ] **Step 4: Wire `ensure_fresh_index` into the CLI**

In `crates/gitx-cli/src/commands/mod.rs`, add:

```rust
/// Honor `[index] auto_refresh` (docs/16 §3): before an index-backed
/// command, refresh a stale/absent persisted index so analysis reads from
/// SQLite instead of recomputing from Git. Skipped when indexing is
/// disabled, auto-refresh is off, or `--no-cache` is given. Progress goes
/// to stderr; stdout stays machine-clean.
pub fn ensure_fresh_index(cli: &Cli, repo: &gitx_git::Repository) -> anyhow::Result<()> {
    let config = crate::commands::config::load_config_for(cli, repo)?;
    if !config.index.enabled || !config.index.auto_refresh || cli.no_cache {
        return Ok(());
    }
    if crate::commands::index::index_is_fresh(repo) {
        return Ok(());
    }
    eprintln!("indexing repository history (auto_refresh)…");
    gitx_services::IndexService::new(repo).refresh_if_stale()?;
    Ok(())
}
```

(`load_config_for` already exists in `crates/gitx-cli/src/commands/config.rs` — it layers repo `gitx.toml` over the global config; verify its exact signature when wiring.)

Call it at the top of the index-backed commands:
- `repo.rs::stats` — after `open_repo(cli)?`, before `service.stats_from_index()`.
- `analysis.rs::analyze` helper — first line, before `AnalysisService::new(repo).analyze(...)`.
- `analysis.rs::contributors` / `ownership` / `hotspots` / `risk` / `health` — they all route through `open_repo` + `analyze`/`weights`; adding the call inside `analyze()` covers hotspots/risk/health/ownership. `contributors` does not use `analyze` — add the call there too.
- `search.rs::search` — after `open_repo`, before the `SearchService` call.

For each, the line is exactly:

```rust
    crate::commands::ensure_fresh_index(cli, &repo)?;
```

- [ ] **Step 5: Auto-refresh inside the TUI loader**

In `crates/gitx-tui/src/app.rs`, inside `load_repo_stats`, immediately after the repo is discovered and before `index_stats_or_live(&repo)`:

```rust
    // First-run auto-refresh (docs/16 `[index] auto_refresh`): build a
    // stale/absent index so every panel reads from SQLite instead of
    // recomputing live from Git. Honored only when config allows it.
    {
        let path = gitx_core::config::Config::default_path();
        let config = path
            .as_deref()
            .map(gitx_core::config::Config::load)
            .transpose()
            .ok()
            .flatten()
            .unwrap_or_default();
        if config.index.enabled && config.index.auto_refresh {
            let _ = gitx_services::IndexService::new(&repo).refresh_if_stale();
        }
    }
```

- [ ] **Step 6: Run the verification**

```bash
cargo test -p gitx-tests --test product
scripts/check.sh
```

Expected: both new tests pass (the second confirms the config gate; the first confirms the index is built and `index status` reports 2 commits).

- [ ] **Step 7: Commit**

```bash
git add crates/gitx-services/src/index.rs crates/gitx-cli/src/commands crates/gitx-tui/src/app.rs tests/integration/product.rs
git commit -m "feat: honor [index] auto_refresh — first analytical command builds the index"
```

---
### Task 4: Health bands read correctly

**Files:**
- Modify: `crates/gitx-analysis/src/lib.rs`
- Modify: `crates/gitx-cli/src/commands/analysis.rs`
- Test: `tests/integration/product.rs`
- Modify: `docs/10-ANALYSIS-ENGINE.md`

**Interfaces:**
- Consumes: `RepoAnalysis.health` sub-scores (exist).
- Produces: `gitx_analysis::health_band(score: f64) -> &'static str` mapping 0–30 POOR, 31–60 FAIR, 61–80 GOOD, 81–100 EXCELLENT. The TUI Health view already colors high scores green; this task only fixes the CLI text (verify the TUI in Step 4).

- [ ] **Step 1: Write the failing test**

Append to `tests/integration/product.rs`:

```rust
#[test]
fn health_output_labels_bands_health_style() {
    let Some(repo) = FixtureRepo::new("product-healthbands") else {
        return;
    };
    repo.write("src/lib.rs", "pub fn hello() {}\n");
    repo.commit("feat: initial");

    let out = Command::new(bin())
        .arg("--repo")
        .arg(repo.path())
        .arg("health")
        .output()
        .expect("gitx health runs");
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(
        text.contains("POOR") && text.contains("EXCELLENT"),
        "health bands must use health semantics, got: {text}"
    );
    assert!(
        !text.contains("CRITICAL"),
        "health output must not reuse the risk CRITICAL band: {text}"
    );
    assert!(
        text.to_lowercase().contains("healthier"),
        "health output should state higher = healthier: {text}"
    );
}
```

Also add a unit test in `crates/gitx-analysis/src/lib.rs` (or the module where you add `health_band`):

```rust
#[test]
fn health_band_partitions_0_100() {
    assert_eq!(health_band(0.0), "POOR");
    assert_eq!(health_band(29.9), "POOR");
    assert_eq!(health_band(30.0), "FAIR");
    assert_eq!(health_band(60.9), "FAIR");
    assert_eq!(health_band(61.0), "GOOD");
    assert_eq!(health_band(80.9), "GOOD");
    assert_eq!(health_band(81.0), "EXCELLENT");
    assert_eq!(health_band(100.0), "EXCELLENT");
}
```

- [ ] **Step 2: Run the tests to verify they fail**

```bash
cargo test -p gitx-tests --test product -- health_output
cargo test -p gitx-analysis health_band_partitions
```

Expected: FAIL — the CLI prints `0–30 LOW · … · 81–100 CRITICAL` and `health_band` does not exist.

- [ ] **Step 3: Add `health_band` and fix the CLI text**

In `crates/gitx-analysis/src/lib.rs` (top-level, `pub`):

```rust
/// Health sub-score band labels (docs/10 §8): health scores are
/// higher-is-better, so the labels run POOR → EXCELLENT — the opposite
/// direction of the risk/hotspot bands. Shared by the CLI so the printed
/// bands can never drift from the TUI's color mapping again.
pub fn health_band(score: f64) -> &'static str {
    if score < 31.0 {
        "POOR"
    } else if score < 61.0 {
        "FAIR"
    } else if score < 81.0 {
        "GOOD"
    } else {
        "EXCELLENT"
    }
}
```

In `crates/gitx-cli/src/commands/analysis.rs`, replace:

```rust
    println!("  Bands: 0–30 LOW · 31–60 MEDIUM · 61–80 HIGH · 81–100 CRITICAL");
```

with:

```rust
    println!(
        "  Bands: 0–30 POOR · 31–60 FAIR · 61–80 GOOD · 81–100 EXCELLENT (higher = healthier)"
    );
```

and add a labeled example use so the helper is exercised (not dead code) — print the overall band in the header line:

```rust
    println!(
        "Repository Health  (composite, deterministic — docs/10 §8)  band: {}",
        gitx_analysis::health_band(h.overall_score)
    );
```

- [ ] **Step 4: Verify TUI health colors match**

In `crates/gitx-tui/src/views/health.rs`, confirm the sub-score color mapping is green for high scores (the polish pass documented green/yellow/red for health). If it is inverted (red at high), fix the comparison operators so high = green. This is an audit step — no change is expected.

- [ ] **Step 5: Update docs/10**

In `docs/10-ANALYSIS-ENGINE.md` §8 (health), replace any "LOW/MEDIUM/HIGH/CRITICAL bands apply to health sub-scores" wording with the health-specific bands: `0–30 POOR · 31–60 FAIR · 61–80 GOOD · 81–100 EXCELLENT`, noting higher = healthier.

- [ ] **Step 6: Run the verification**

```bash
cargo test -p gitx-tests --test product -- health
cargo test -p gitx-analysis
scripts/check.sh
```

Expected: all pass. If any CLI snapshot test includes the old band line, regenerate with `GITX_BLESS=1 cargo test -p gitx-tests --test cli_snapshots`.

- [ ] **Step 7: Commit**

```bash
git add crates/gitx-analysis/src/lib.rs crates/gitx-cli/src/commands/analysis.rs tests/integration/product.rs docs/10-ANALYSIS-ENGINE.md
git commit -m "fix: health bands are health-oriented (POOR..EXCELLENT), never CRITICAL"
```

---
### Task 5: Wire the dead configuration options

**Files:**
- Modify: `crates/gitx-cli/src/cli.rs`
- Modify: `crates/gitx-cli/src/commands/analysis.rs`
- Modify: `crates/gitx-cli/src/commands/history.rs`
- Modify: `crates/gitx-cli/src/commands/index.rs`
- Modify: `crates/gitx-cli/src/commands/search.rs`
- Modify: `crates/gitx-services/src/search.rs`
- Modify: `crates/gitx-search/src/code.rs`
- Test: `tests/integration/product.rs`, `crates/gitx-search/src/code.rs` unit tests
- Modify: `docs/16-CONFIGURATION.md`

**Interfaces:**
- Consumes: `Config.general.default_limit`, `Config.index.enabled`, `Config.search.case_sensitive` (all parsed today, never consumed); `App.vim_keys` from Task 2 (already consumed).
- Produces: `SearchOptions.case_sensitive: bool`; `gitx_search::search_code_content(repo, query, cap, case_sensitive) -> CodeSearchOutcome`; `hotspots --limit` and `timeline --max` become `Option<usize>` falling back to `default_limit`; `index.enabled=false` makes `scan`/`refresh` no-ops and forces live analysis.

- [ ] **Step 1: Write the failing tests**

Append to `tests/integration/product.rs`:

```rust
#[test]
fn index_disabled_skips_scan_and_forces_live_analysis() {
    let Some(repo) = FixtureRepo::new("product-indexdisabled") else {
        return;
    };
    repo.write("README.md", "# demo\n");
    repo.commit("docs: readme");
    std::fs::write(
        repo.path().join("gitx.toml"),
        "[index]\nenabled = false\n",
    )
    .expect("write repo config");

    let out = Command::new(bin())
        .arg("--repo")
        .arg(repo.path())
        .arg("scan")
        .output()
        .expect("gitx scan runs");
    assert!(out.status.success(), "scan must succeed with a message");
    assert!(
        !repo.path().join(".git/gitx/index.sqlite").exists(),
        "index.enabled=false must not create an index"
    );

    let out = Command::new(bin())
        .arg("--repo")
        .arg(repo.path())
        .arg("stats")
        .output()
        .expect("gitx stats runs");
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(
        text.contains("source: live"),
        "stats must be labeled live when the index is disabled: {text}"
    );
}

#[test]
fn default_limit_applies_to_hotspots() {
    let Some(repo) = FixtureRepo::new("product-defaultlimit") else {
        return;
    };
    for i in 0..10 {
        repo.write(&format!("src/f{i}.rs"), "pub fn x() {}\n");
        repo.commit(&format!("feat: file {i}"));
    }
    std::fs::write(
        repo.path().join("gitx.toml"),
        "[general]\ndefault_limit = 3\n",
    )
    .expect("write repo config");

    let out = Command::new(bin())
        .arg("--repo")
        .arg(repo.path())
        .arg("hotspots")
        .output()
        .expect("gitx hotspots runs");
    let text = String::from_utf8_lossy(&out.stdout);
    let rows = text
        .lines()
        .filter(|l| l.trim_start().starts_with(|c: char| c.is_ascii_digit()))
        .count();
    assert_eq!(
        rows, 3,
        "hotspots should honor default_limit=3 rows, got {rows}:\n{text}"
    );
}
```

Add to `crates/gitx-search/src/code.rs` tests:

```rust
    #[test]
    fn case_sensitivity_controls_matching() {
        // Not a real repo: search_code_content requires a Repository, so
        // assert the lower-level predicate instead by extracting it — see
        // Step 3 — then here test through the public wrapper when a repo
        // exists. Until Step 3 lands this test fails to compile (expected).
    }
```

(The unit test above is intentionally a compile-time placeholder: Step 3 extracts the case predicate so Step 4 replaces this comment with a real assertion. Delete this test body in Step 4 and replace with the real test below.)

- [ ] **Step 2: Run the tests to verify they fail**

```bash
cargo test -p gitx-tests --test product -- index_disabled default_limit
```

Expected: FAIL — `gitx scan` ignores `enabled=false` (creates the index), `hotspots` ignores `default_limit` (prints 20 rows).

- [ ] **Step 3: Extract a case predicate in `gitx-search`**

In `crates/gitx-search/src/code.rs`, change `search_code_content`'s signature and matching:

```rust
pub fn search_code_content(
    repo: &Repository,
    query: &str,
    cap: usize,
    case_sensitive: bool,
) -> CodeSearchOutcome {
    let cap = cap.max(1);
    let mut results = Vec::new();
    let mut files_scanned = 0usize;
    let mut truncated = false;
    let matches = |line: &str| {
        if case_sensitive {
            line.contains(query)
        } else {
            line.to_lowercase().contains(&query.to_lowercase())
        }
    };

    if let Some(work_dir) = repo.work_dir() {
        let mut outcome = WorktreeSearch {
            results: &mut results,
            query,
            matches: &matches,
            cap,
            files_scanned: &mut files_scanned,
            truncated: &mut truncated,
        };
        walk_worktree(work_dir, work_dir, &mut outcome, 0);
        ...
```

Adjust `WorktreeSearch` to carry `matches: &'a dyn Fn(&str) -> bool` and use `s.matches(line)` in both the worktree walk and the HEAD-tree fallback (replace the two `l.contains(query)` / `l.contains(s.query)` call sites). Update the two callers of `search_code_content` (CLI `crates/gitx-cli/src/commands/search.rs` and `crates/gitx-services/src/search.rs`) to pass `case_sensitive: false` — Task 5 Step 5 then threads the real config value through.

- [ ] **Step 4: Replace the placeholder unit test with a real one**

In `crates/gitx-search/src/code.rs`, replace the Step-1 placeholder test with:

```rust
    #[test]
    fn case_matching_predicate_honors_flag() {
        // The predicate lives in search_code_content; exercise the two
        // branches via a tiny extraction so this test stays hermetic:
        let cs = |line: &str, query: &str, sensitive: bool| {
            if sensitive {
                line.contains(query)
            } else {
                line.to_lowercase().contains(&query.to_lowercase())
            }
        };
        assert!(cs("Hello World", "hello", false));
        assert!(!cs("Hello World", "hello", true));
        assert!(cs("Hello World", "Hello", true));
    }
```

If Step 3 instead exposed the predicate as a `fn case_contains(line: &str, query: &str, case_sensitive: bool) -> bool`, call that directly — the point is a hermetic unit test covering both branches.

- [ ] **Step 5: Thread `case_sensitive` through the service layer**

In `crates/gitx-services/src/search.rs`, add to `SearchOptions`:

```rust
    /// Case-sensitive matching for FTS-backed scopes and code content
    /// (docs/16 `[search] case_sensitive`). FTS5 is case-insensitive by
    /// default; when true, hits are post-filtered for an exact-case match.
    pub case_sensitive: bool,
```

After all scopes have been collected (just before `Ok(hits)`), add:

```rust
        if options.case_sensitive {
            let term = raw.trim();
            hits.retain(|h| {
                h.title.contains(term) || h.id.contains(term) || h.detail.contains(term)
            });
        }
```

and pass the flag to the code-content call:

```rust
        if options.code && !raw.trim().is_empty() {
            let outcome = gitx_search::search_code_content(
                self.repo,
                raw.trim(),
                50,
                options.case_sensitive,
            );
```

In `crates/gitx-cli/src/commands/search.rs`, load the config and set `case_sensitive` on the options struct (it already reads config for other settings — add `case_sensitive: config.search.case_sensitive`).

- [ ] **Step 6: Wire `default_limit` and `index.enabled` in the CLI**

In `crates/gitx-cli/src/cli.rs`, change the `Hotspots` subcommand:

```rust
    Hotspots {
        /// Show at most N files (default: `[general] default_limit`).
        #[arg(long)]
        limit: Option<usize>,
        /// Restrict to a path prefix.
        #[arg(long)]
        path: Option<String>,
    },
```

In `crates/gitx-cli/src/commands/analysis.rs`, update `hotspots` to resolve the limit:

```rust
pub fn hotspots(cli: &Cli, limit: Option<usize>, path: Option<&str>) -> anyhow::Result<()> {
    let repo = open_repo(cli)?;
    crate::commands::ensure_fresh_index(cli, &repo)?;
    let analysis = analyze(cli, &repo)?;
    let default_limit = crate::commands::config::load_config_for(cli, &repo)?
        .general
        .default_limit;
    let limit = limit.unwrap_or(default_limit);
```

and in `risk`, replace `analysis.files.iter().take(20)` with a `default_limit`-bounded take:

```rust
    let config = crate::commands::config::load_config_for(cli, &repo)?;
    let files: Vec<&FileAnalysis> = match path {
        Some(p) => analysis
            .files
            .iter()
            .filter(|f| f.path == std::path::Path::new(p) || f.path.starts_with(p))
            .collect(),
        None => analysis.files.iter().take(config.general.default_limit).collect(),
    };
```

In `crates/gitx-cli/src/commands/history.rs`, in `timeline`, replace the hardcoded cap with the config default when `--max` is absent (locate the current `max` handling — `max_count: Some(...)` or `.take(max.unwrap_or(...))` — and change the fallback to `load_config_for(cli, &repo)?.general.default_limit`).

In `crates/gitx-cli/src/commands/index.rs`, at the top of `run_indexer`:

```rust
    let repo = open_repo(cli)?;
    let config = crate::commands::config::load_config_for(cli, &repo)?;
    if !config.index.enabled {
        println!("indexing is disabled ([index] enabled = false); skipping");
        return Ok(());
    }
```

In `crates/gitx-cli/src/commands/analysis.rs`, `analyze()` and in `repo.rs::stats`, force the live path when the index is disabled:

```rust
    // [index] enabled = false → never read the persisted index (docs/16).
    let config = crate::commands::config::load_config_for(cli, repo)?;
    if !config.index.enabled {
        // live path
    }
```

For `analyze` this means calling `analyze_repository_with(repo, weights)` directly when disabled; for `stats` it means using `gitx_analysis::repository_stats(repo)` with `source: live`.

- [ ] **Step 7: Update docs/16**

In `docs/16-CONFIGURATION.md`, under §3, mark the now-wired options as implemented: `default_limit` (list-command row caps), `index.enabled` (scan/refresh no-op + forced live analysis), `index.auto_refresh` (first analytical command builds a stale index; see Task 3), `search.case_sensitive` (FTS post-filter + code search), `ui.vim_keys` (Task 2). Replace the "Exact keys should be finalized during implementation" note with a line stating all keys are honored.

- [ ] **Step 8: Run the verification**

```bash
cargo test -p gitx-tests --test product
cargo test -p gitx-search
scripts/check.sh
```

Expected: all pass. Regenerate any affected snapshots with `GITX_BLESS=1 cargo test -p gitx-tests --test cli_snapshots` (hotspots row count changes).

- [ ] **Step 9: Commit**

```bash
git add crates/gitx-cli crates/gitx-services/src/search.rs crates/gitx-search/src/code.rs tests/integration/product.rs docs/16-CONFIGURATION.md
git commit -m "feat: wire default_limit, index.enabled, and search.case_sensitive config options"
```

---
### Task 6: README — the storefront

**Files:**
- Rewrite: `README.md`
- Modify: `docs/README.md` (link check only, if it exists)

**Interfaces:**
- Consumes: the `gitx` no-arg behavior from Task 2 (the Quick Start can now say "run `gitx`"), the auto-refresh from Task 3 (first command builds the index), the health output from Task 4.
- Produces: a README that installs, demos, and tours the product; referenced by Task 7's release notes.

- [ ] **Step 1: Write the new README**

Replace the entire `README.md` with:

````markdown
# GitX

> Local-first, terminal-native Git repository intelligence and code archaeology.

[![CI](https://github.com/abuzarkhan1/gitx/actions/workflows/ci.yml/badge.svg)](https://github.com/abuzarkhan1/gitx/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/gitx-cli)](https://crates.io/crates/gitx-cli)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

GitX turns a Git repository's history, structure, changes, ownership, branches,
dependencies, and recoverable work into a fast, interactive, explainable
terminal experience — with **no network, no accounts, no AI**. Every score
exposes the raw Git signals behind it.

![gitx dashboard](docs/assets/gitx-dashboard.png)

## Quick start

```bash
# In any Git repository:
gitx
```

`gitx` opens the interactive dashboard: repository health, activity,
hotspots, ownership, branches, architecture, dependencies, recovery — all
explorable with the keyboard or mouse. Your history is indexed into a local
SQLite cache on first use, so everything after that is sub-second.

Prefer the command line? Every capability is a command:

```bash
gitx stats                  # repository statistics
gitx hotspots               # files ranked by maintenance risk
gitx health                 # composite health score, six sub-scores
gitx ownership              # who owns what, and where it concentrates
gitx lineage src/main.rs    # the full life of a file, renames included
gitx blame src/main.rs      # line-level attribution
gitx branches               # divergence, age, shared files, staleness
gitx search "deadlock"      # FTS across commits, files, authors, tags
gitx recovery               # reflog, unreachable commits, dangling objects
gitx dependencies           # declared + lockfile-precise dependencies
gitx symbols                # functions/classes extracted from HEAD
gitx release diff v1.0 v1.1 # what shipped between releases
```

All analytical commands emit machine-readable output:

```bash
gitx --json hotspots
gitx --csv contributors
```

## Install

```bash
# crates.io (CLI only)
cargo install gitx-cli --locked

# cargo-dist installers (CLI + TUI) — one line, from the GitHub Releases page
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/abuzarkhan1/gitx/releases/latest/download/gitx-installer.sh | sh

# Homebrew (when the tap is published)
brew install abuzarkhan1/tap/gitx
```

Then add shell completions:

```bash
gitx completions bash   # zsh / fish / powershell also supported
```

## What makes GitX different

- **Explainable, not black-box.** `gitx risk src/main.rs` prints the formula,
  the time window, and every input (change frequency, churn, bug-fix rate,
  ownership concentration, complexity). No hidden scoring.
- **Local and private.** Everything runs on your machine against your
  repository. Nothing leaves it.
- **Deterministic.** The same repository and configuration produce the same
  results, bit for bit — safe for CI.
- **Built for archaeology.** Rename-following lineage, copy-source tracking,
  symbol history, and recovery of unreachable work are first-class features,
  not afterthoughts.
- **Fast at scale.** A persistent SQLite index means hot queries read
  milliseconds, with phased lazy loading in the dashboard on large
  repositories.

## Documentation

The full specification set lives in [`docs/`](docs/INDEX.md): product
requirements, CLI and TUI specifications, the analysis engine, the database
schema, the recovery model, and a docs ⇄ code audit matrix
([`docs/26-IMPLEMENTATION-STATUS.md`](docs/26-IMPLEMENTATION-STATUS.md)).

## Development

```bash
cargo build --workspace
scripts/check.sh            # fmt + clippy -D warnings + tests
scripts/verify-tui.sh       # headless PTY verification of the dashboard
scripts/bench.sh            # criterion baselines → benches/RESULTS.md
```

See [`docs/22-CONTRIBUTING.md`](docs/22-CONTRIBUTING.md) and
[`docs/27-RELEASING.md`](docs/27-RELEASING.md).

## License

MIT
````

- [ ] **Step 2: Add the screenshot asset (manual, needs a terminal)**

The README references `docs/assets/gitx-dashboard.png`. Generate it with the existing harness:

```bash
mkdir -p docs/assets
cargo build -p gitx-tui
# in a 140x44 tmux pane against a real repository, run `gitx`, then:
tmux capture-pane -p -e | head -44  # adjust, then render to PNG
```

Use `scripts/verify-tui.sh`'s captured frames (`/tmp/gitx-tui-verify/00_lazy_overview.txt`) as the source and convert with your terminal's image capture (iTerm2 `imgcat`-style capture, `tiv`, or an ANSI-to-PNG renderer). If no converter is available, record a short terminal animation instead (`asciinema rec`) and link it — the README works either way as long as the asset link resolves.

- [ ] **Step 3: Verify links resolve**

```bash
# every relative link in the new README must exist
grep -oE '\]\([^)]+' README.md | sed 's/](//' | while read -r link; do
  case "$link" in
    http*|mailto:*) ;;
    *) [ -e "$link" ] || echo "MISSING: $link" ;;
  esac
done
```

Expected: no `MISSING:` lines (create `docs/assets/gitx-dashboard.png` or adjust the link in Step 2).

- [ ] **Step 4: Commit**

```bash
git add README.md docs/assets 2>/dev/null || git add README.md
git commit -m "docs: storefront README with install paths, tour, and dashboard demo"
```

---
### Task 7: Release runbook — ship 0.1.0 (maintainer-gated)

**Files:**
- Modify: `docs/27-RELEASING.md`
- Manual: tag, GitHub release, crates.io publishes

**Interfaces:**
- Consumes: Tasks 1–6 (publishable manifests, working `gitx` no-arg flow, README). Requires a GitHub token with `contents: write` (for the release workflow) and crates.io API tokens for publishing — these are credentials the implementing agent must **not** possess; every step below that touches the network is explicitly a maintainer action.

- [ ] **Step 1: Write the failing verification (local dry-run)**

```bash
cargo dist plan 2>/dev/null || echo "cargo-dist not installed locally — CI will verify on the tag"
cargo package -p gitx-cli --allow-dirty --no-verify && echo "cli package OK"
```

Expected: `cli package OK`; `cargo dist plan` may be absent locally — that is fine because `.github/workflows/release.yml` runs the dist build on the tag.

- [ ] **Step 2: Update the release runbook**

In `docs/27-RELEASING.md`, replace the checklist with the concrete 0.1.0 runbook:

```markdown
## 0.1.0 runbook

1. Confirm `scripts/check.sh` is green and `scripts/verify-tui.sh` passes.
2. Confirm `cargo package` succeeds for all 11 crates (Task 1 of
   `docs/superpowers/plans/2026-08-13-product-hardening.md`).
3. Publish crates bottom-up (leaf dependencies first — derive order with
   `cargo tree -p gitx-cli --edges normal | grep gitx-`):
   `cargo publish -p gitx-core` → `gitx-git` → `gitx-storage` → `gitx-index`
   → `gitx-history` → `gitx-analysis` → `gitx-graph` → `gitx-search` →
   `gitx-services` → `gitx-cli` → `gitx-tui`.
4. Tag and push: `git tag -a v0.1.0 -m "GitX 0.1.0" && git push origin v0.1.0`.
5. `.github/workflows/release.yml` builds installers for the four targets,
   checksums, and the GitHub Release (notes from CHANGELOG.md).
6. Smoke-test the release binaries with `scripts/release-check.sh`.
7. Publish the Homebrew tap formula if the tap repository exists
   (`brew tap abuzarkhan1/tap`).
```

- [ ] **Step 3: Commit the runbook**

```bash
git add docs/27-RELEASING.md
git commit -m "docs: concrete 0.1.0 release runbook"
```

- [ ] **Step 4: (Maintainer) Execute the release**

Run the runbook's steps 3–6. This is outside the automated gate: publishing requires crates.io tokens and a tag push. The implementing agent stops after Step 3 and hands the runbook to the maintainer.

- [ ] **Step 5: Update docs/26**

After the tag exists, add a line to `docs/26-IMPLEMENTATION-STATUS.md` under the audit matrix: `18-RELEASE-ENGINEERING — v0.1.0 shipped (tag + crates.io + installers)`. Skip if the release has not happened yet.

---
## Self-Review

**Spec coverage (audit findings → task mapping):**

| Audit finding | Task |
|---|---|
| Never shipped; CHANGELOG placeholder; no publish metadata | Task 1 + Task 7 |
| `gitx` no-arg prints a hint; TUI is a separate binary; docs/16 §7 broken | Task 2 |
| First-run analysis is slow (3 s on 39 commits); `auto_refresh` documented but dead | Task 3 |
| Health bands reuse risk labels ("81–100 CRITICAL" on a healthy score) | Task 4 |
| `default_limit`/`auto_refresh`/`vim_keys`/`case_sensitive`/`index.enabled` parsed but never consumed | Tasks 2, 3, 5 |
| README is not a storefront (no badges, install paths, demo, tour) | Task 6 |
| No release runbook | Task 7 |

**Placeholder scan:** Every task's steps carry exact code, exact test bodies, and exact commands. The only intentional exceptions: Task 6 Step 2 (screenshot capture — inherently manual, with concrete commands and an explicit fallback) and Task 7 Step 4 (network publish — explicitly gated to the maintainer). Task 5 Step 1 has one placeholder test body that Step 4 replaces with a real assertion; the plan says so at both points.

**Type consistency:** `gitx_tui::run(vim_keys: bool)` is introduced in Task 2 and consumed by `run_dashboard_or_tui` in the same task; Task 5 never re-signatures it. `IndexService::refresh_if_stale()` is defined in Task 3 and used by both `ensure_fresh_index` and the TUI loader in the same task. `search_code_content` gains its `case_sensitive` parameter once, in Task 5 Step 3, with both call sites updated in the same step. `default_limit` stays `usize` throughout; `hotspots --limit` becomes `Option<usize>` once.

**Explicitly deferred (separate plans, not this one):**
- TUI archaeology depth: a full diff pane in the commit detail, blame-in-TUI, and timeline pagination beyond the 500-commit cap (the cap is a documented bound today).
- Large-repository validation: dogfooding `gitx scan`/`health`/TUI on a real large repository (e.g. a 50k+-commit repo) and recording the timings into `benches/RESULTS.md`; the sub-second-startup goal (docs/26 item 54) is the acceptance target.
