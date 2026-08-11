# GitX Documentation

GitX is a **local-first, terminal-native Git repository intelligence and code archaeology CLI**.

It is not a SaaS product, web application, cloud service, collaboration platform, or AI assistant. The project is intentionally focused on one thing:

> Turn a Git repository's history, structure, changes, ownership, branches, dependencies, and recovery information into a fast, interactive, explainable terminal experience.

## Product principles

1. **CLI-first** — the terminal is the primary product surface.
2. **Local-first** — repository analysis happens locally.
3. **No AI dependency** — every insight is derived from deterministic repository data and documented algorithms.
4. **Explainable intelligence** — every score or warning must expose the signals behind it.
5. **Incremental by default** — avoid rescanning unchanged history.
6. **Fast interactive UX** — the TUI should feel responsive even on large repositories.
7. **Git-native** — use Git's actual object model rather than treating Git as a collection of shell commands.
8. **Machine-readable** — analytical commands support structured JSON output.
9. **Cross-platform** — target macOS, Linux, and Windows.
10. **Modular architecture** — Git parsing, indexing, analysis, storage, CLI, and TUI remain separable.

## Documentation map

- `01-PRD.md` — product requirements and boundaries
- `02-PRODUCT-SCOPE.md` — in-scope, out-of-scope, MVP and later capabilities
- `03-TECH-STACK.md` — technology decisions and rationale
- `04-SYSTEM-ARCHITECTURE.md` — high-level and internal architecture
- `05-DOMAIN-MODEL.md` — repository intelligence data model
- `06-DATABASE-SCHEMA.md` — SQLite/index schema
- `07-CLI-SPECIFICATION.md` — commands, flags, output contracts
- `08-TUI-SPECIFICATION.md` — terminal UI information architecture and interaction model
- `09-INDEXING-ENGINE.md` — initial and incremental indexing design
- `10-ANALYSIS-ENGINE.md` — hotspots, ownership, risk, architecture, metrics
- `11-SEARCH-SPECIFICATION.md` — search and filtering design
- `12-RECOVERY-SPECIFICATION.md` — reflog and unreachable-object analysis
- `13-PERFORMANCE-ENGINEERING.md` — performance budgets and optimization strategy
- `14-TESTING-STRATEGY.md` — test pyramid, fixtures, snapshots and benchmarks
- `15-SECURITY-PRIVACY.md` — local-first security and privacy requirements
- `16-CONFIGURATION.md` — configuration, cache and environment behavior
- `17-IMPLEMENTATION-PLAN.md` — phased implementation plan
- `18-RELEASE-ENGINEERING.md` — packaging, distribution and release process
- `19-QUALITY-GATES.md` — definition of done and release gates
- `20-ADR.md` — architecture decision records
- `21-ROADMAP.md` — staged roadmap
- `22-CONTRIBUTING.md` — development workflow and contribution rules

## Source of truth

The implementation must follow these documents together. If implementation discovers a conflict, update the relevant ADR and specification before introducing behavior that contradicts the documented design.
