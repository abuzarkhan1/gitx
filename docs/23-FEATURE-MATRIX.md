# Feature Matrix

| Area | Feature | MVP | TUI | CLI | JSON | Persistent Index |
|---|---|---:|---:|---:|---:|---:|
| Repository | Overview | Yes | Yes | Yes | Yes | Yes |
| Repository | Repository age | Yes | Yes | Yes | Yes | Yes |
| Repository | Language breakdown | Yes | Yes | Yes | Yes | Yes |
| History | Timeline | Yes | Yes | Yes | Yes | Yes |
| History | Commit details | Yes | Yes | Yes | Yes | Yes |
| History | File history | Yes | Yes | Yes | Yes | Yes |
| History | Rename lineage | Yes | Yes | Yes | Yes | Yes |
| History | Copy-source lineage | Yes | Yes | Yes | Yes | Yes |
| History | Symbol history | Yes | — | Yes | Yes | Yes |
| History | Line-level history (blame) | Later | Yes | Yes | Yes | Yes |
| History | Diff output (`gitx diff`) | Later | — | Yes | — | — |
| Branches | Branch intelligence | Yes | Yes | Yes | Yes | Yes |
| People | Contributors | Yes | Yes | Yes | Yes | Yes |
| People | Ownership | Yes | Yes | Yes | Yes | Yes |
| Analysis | Hotspots | Yes | Yes | Yes | Yes | Yes |
| Analysis | Complexity signal (function count) | Yes | Yes | Yes | Yes | Yes |
| Analysis | Risk | Yes | Yes | Yes | Yes | Yes |
| Analysis | Repository health | Yes | Yes | Yes | Yes | Yes |
| Analysis | Bug/regression history | Yes | Yes | Yes | Yes | Yes |
| Architecture | Directory evolution | Yes | Yes | Yes | Yes | Yes |
| Architecture | Dependency evolution | Yes | Yes | Yes | Yes | Yes |
| Search | Full-text history | Yes | Yes | Yes | Yes | Yes |
| Search | Rename search (`--renames`) | Yes | Yes | Yes | Yes | Yes |
| Search | Symbol search | Yes | Yes | Yes | Yes | Yes |
| Search | Directory search | Yes | Yes | Yes | Yes | Yes |
| Search | Code-content search (`--code`) | Yes | Yes | Yes | Yes | — |
| Export | CSV output (`--csv`) | Later | — | Yes | — | — |
| Recovery | Reflog | Yes | Yes | Yes | Yes | Yes |
| Recovery | Unreachable commits | Yes | Yes | Yes | Yes | Yes |
| Recovery | Dangling trees/blobs | Yes | Yes | Yes | Yes | Yes |
| Release | Ref comparison | Yes | Yes | Yes | Yes | Yes |
| Structure | Language symbols | Yes | Yes | Yes | Yes | Yes |
| Structure | Symbol history | Yes | — | Yes | Yes | Yes |
| Structure | Call graph (`gitx graph`, TUI Graph) | Yes | Yes | Yes | Yes | Yes |
| Structure | Tree-sitter analysis | Later | — | — | — | — |

Notes:

- Language symbols use the deterministic line-based extractor
  (`gitx symbols`, docs/21 Stage 6) — **not** Tree-sitter. Tree-sitter is
  deferred (ADR-011, Accepted-deferred, with revisit criteria); the former
  `gitx-graph` parser placeholder was removed because nothing consumed it.
  Call edges in `gitx graph` are heuristic (`name(` scans) and feed both
  the CLI and the TUI Graph view; the hotspot/risk score uses the symbol
  extractor's function count as its complexity signal (`symbols+loc`),
  falling back to labeled LOC.
- `gitx symbols history <name>` walks commit lineage to locate a symbol's
  birth, moves, renames, and deletion; the TUI does not expose it (CLI
  only). `gitx diff A B` streams bounded unified output with a `--stat`
  mode; neither is part of the persisted index.
- CSV export (`--csv`) covers the tabular commands (hotspots, risk,
  health, branches, contributors, ownership, timeline) with deterministic
  quoting via `gitx-core::csv`.
- Code-content search is bounded to the working tree with a HEAD-tree
  fallback (docs/11 §5) and is not part of the persisted index.


## Explicitly excluded

| Capability | Status |
|---|---|
| AI | Excluded |
| SaaS | Excluded |
| Web application | Excluded |
| Cloud storage | Excluded |
| Accounts | Excluded |
| Authentication | Excluded |
| Collaboration | Excluded |
| Hosted API | Excluded |
| Project management | Excluded |
| Issue tracker | Excluded |
| CI/CD platform | Excluded |
