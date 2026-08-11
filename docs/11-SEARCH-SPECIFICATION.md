# Search Specification

## 1. Objective

Search should make repository history queryable without forcing the user to remember Git command syntax.

## 2. Searchable entities

- commit message
- commit OID
- author
- committer
- file path
- file rename history (old and new paths)
- branch
- tag
- symbol
- directory
- code content (current working tree; optionally indexed snapshots)

## 3. Basic

```bash
gitx search "workspace"
```

## 4. Filters

```bash
gitx search "workspace" --commits
gitx search "workspace" --files
gitx search "workspace" --authors
gitx search "workspace" --branches
gitx search "workspace" --renames
gitx search "workspace" --code
gitx search "workspace" --history
gitx search "workspace" --since 2026-01-01
gitx search "workspace" --author Abuzar
```

- `--renames` searches rename history (old and new paths)
- `--code` searches file contents
- `--history` scopes to historical records (commit messages, renames, past paths)

## 5. Search behavior

Search should support:

- exact terms
- token matching
- prefix matching where appropriate
- path filtering
- date filtering
- author filtering
- ref filtering

Code-content search is bounded to the current working tree and optionally indexed snapshots; it must never stream every historical blob.

## 6. FTS

SQLite FTS5 should provide the primary text-search implementation.

Do not make raw SQL part of the public CLI.

## 7. Search results

Example:

```text
SEARCH: workspace

Commits (24)
  a81f92c  feat: add workspace persistence
  72c91ad  refactor: workspace state

Files (13)
  src/workspace/WorkspaceManager.rs
  src/workspace/state.rs

Branches (3)
  feature/workspaces
  refactor/workspace-state
```

## 8. Ranking

Results should prioritize:

1. exact matches
2. path/name matches
3. recent matches
4. high-relevance textual matches

Ranking must remain deterministic.

## 9. JSON

JSON results should include:

```text
entity_type
id
display_name
match_context
score_if_applicable
```
