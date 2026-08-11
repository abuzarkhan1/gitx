# Implementation Plan

## Phase 0 — Repository bootstrap

### Tasks

- initialize Rust workspace
- configure workspace crates
- configure formatting
- configure clippy
- configure CI
- establish license
- establish contribution rules
- establish error conventions
- establish logging conventions

### Deliverable

Compiling multi-crate workspace with CI.

---

## Phase 1 — Git foundation

### Tasks

- repository discovery
- Git repository abstraction
- object ID type
- commit model
- parent model
- author model
- refs
- branches
- tags
- tree access
- diff/stat extraction
- repository state

### Deliverable

A library can inspect a repository without TUI.

---

## Phase 2 — Storage

### Tasks

- SQLite connection layer
- migrations
- repository metadata
- commit persistence
- parent persistence
- file persistence
- change persistence
- branch/tag persistence
- author persistence
- indexes
- transaction helpers

### Deliverable

A complete local repository index can be persisted and reopened.

---

## Phase 3 — Initial indexer

### Tasks

- initial traversal
- normalization
- batch inserts
- progress reporting
- cancellation
- index metadata
- consistency checks

### Deliverable

```bash
gitx scan
```

creates a correct index.

---

## Phase 4 — Incremental indexer

### Tasks

- detect HEAD changes
- detect branch changes
- detect tag changes
- detect ref rewrites
- calculate affected commits
- update derived data
- invalidation
- atomic refresh

### Deliverable

```bash
gitx refresh
```

updates only necessary data.

---

## Phase 5 — CLI foundation

### Tasks

Implement:

```text
info
status
stats
scan
refresh
timeline
commit
history
branches
contributors
```

Add human output and JSON output.

### Deliverable

GitX is useful without TUI.

---

## Phase 6 — TUI foundation

### Tasks

- terminal initialization
- event loop
- app state
- navigation
- layout
- status bar
- keymap
- loading states
- error dialogs
- overview page

### Deliverable

```bash
gitx
```

opens a usable TUI.

---

## Phase 7 — History explorer

### Tasks

- timeline
- commit graph
- commit detail
- changed-file list
- file detail
- path history
- rename lineage
- line-level history (blame)

### Deliverable

Historical archaeology workflow.

---

## Phase 8 — Analysis engine

### Tasks

- change frequency
- churn
- recency
- bug-fix classification
- contributor count
- ownership
- hotspot score
- risk score
- repository health score

### Deliverable

```bash
gitx hotspots
gitx ownership
gitx risk
```

---

## Phase 9 — Branch intelligence

### Tasks

- ahead/behind
- merge base
- divergence
- branch age
- stale branch detection
- shared-file analysis
- merge complexity estimate

### Deliverable

```bash
gitx branches
```

with meaningful branch analysis.

---

## Phase 10 — Search

### Tasks

- FTS5
- commit search
- path search
- author search
- branch/tag search
- filters
- ranking
- JSON results

### Deliverable

Fast repository-wide search.

---

## Phase 11 — Recovery

### Tasks

- reflog parsing
- unreachable commits
- dangling objects
- deleted branch detection
- recovery presentation
- patch export

### Deliverable

Read-only recovery intelligence.

---

## Phase 12 — Architecture and dependencies

### Tasks

- directory snapshots
- dependency manifests
- dependency history
- structural comparisons
- graph representation
- optional Tree-sitter abstraction

### Deliverable

Architecture evolution view.

---

## Phase 13 — Quality and scale

### Tasks

- benchmark suites
- large repository testing
- memory profiling
- query optimization
- TUI performance optimization
- error hardening
- snapshot stabilization

### Deliverable

Production-quality developer tool.

---

## Phase 14 — Distribution

### Tasks

- release CI
- cargo-dist
- macOS builds
- Linux builds
- Windows builds
- checksums
- versioning
- release notes
- installation documentation

### Deliverable

Installable GitX releases.

---

## Implementation order rule

Do not begin with the TUI.

The correct order is:

```text
Git model
→ storage
→ indexing
→ domain services
→ CLI
→ TUI
→ advanced analysis
```

This prevents presentation logic from becoming the application's architecture.
