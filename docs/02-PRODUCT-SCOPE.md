# Product Scope

## 1. Scope model

GitX has four capability layers:

1. Git data access
2. Repository indexing
3. Deterministic intelligence
4. Terminal presentation

## 2. In scope

### Git understanding

- commits
- parents
- merges
- branches
- tags
- authors/committers
- file changes
- additions/deletions
- renames
- copies where reliably detectable
- reflog
- unreachable objects
- repository state
- line-level history (blame)

### Repository intelligence

- activity
- churn
- change frequency
- file age
- contributor distribution
- ownership concentration
- hotspots
- maintenance-risk indicators
- branch divergence
- stale branches
- architecture evolution
- dependency events
- release comparisons
- recurring bug-fix areas
- regression and recurring problem areas
- repository health scoring
- knowledge concentration / bus factor
- inactive ownership
- dependency usage and churn

### User interfaces

- interactive TUI
- non-interactive CLI
- JSON output
- help/completion

## 3. Out of scope

- cloud storage
- remote dashboards
- accounts
- authentication
- web UI
- hosted APIs
- AI
- LLM integration
- chat
- collaboration
- issue tracking
- pull-request management
- project management

## 4. MVP versus later

### MVP

- Git object/history model
- SQLite index
- TUI shell
- timeline
- commit inspection
- file history
- branch intelligence
- contributors
- hotspots
- ownership
- repository health
- search
- recovery
- basic architecture/dependency analysis

### V1

- stronger architecture graph
- language-aware symbols
- Tree-sitter adapters
- line-level history (blame)
- release comparison
- advanced filters
- richer metrics
- improved recovery diagnostics

### V2

- additional language analyzers
- advanced copy/rename lineage
- more sophisticated dependency extraction
- large-repository performance tuning
- richer export formats if justified

## 5. Feature priority

| Feature | Priority |
|---|---|
| Git engine | P0 |
| Incremental index | P0 |
| TUI shell | P0 |
| Timeline | P0 |
| Commit explorer | P0 |
| File archaeology | P0 |
| Search | P0 |
| Hotspots | P0 |
| Ownership | P0 |
| Repository health | P0 |
| Branch intelligence | P0 |
| Recovery | P0 |
| JSON output | P0 |
| Architecture evolution | P1 |
| Dependency evolution | P1 |
| Line-level history (blame) | P1 |
| Language-aware symbols | P1 |
| Advanced structural analysis | P2 |

## 6. Product boundary rule

Before adding a feature, ask:

> Does this help a developer understand Git history, repository structure, code evolution, ownership, risk, branches, dependencies, or recoverability?

If not, it should not enter the core product without an explicit scope decision.
