# SQLite Database Schema

## 1. Purpose

SQLite is a local derived index. The Git repository remains the source of truth.

Deleting the index must never damage the repository.

## 2. Metadata

```sql
CREATE TABLE index_metadata (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
```

Recommended keys:

```text
schema_version
git_head
indexed_at
repository_id
tool_version
index_format_version
```

## 3. Repository

```sql
CREATE TABLE repositories (
    id INTEGER PRIMARY KEY,
    root_path TEXT NOT NULL UNIQUE,
    git_dir TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
```

## 4. Authors

```sql
CREATE TABLE authors (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT,
    normalized_name TEXT,
    normalized_email TEXT
);
```

## 5. Commits

```sql
CREATE TABLE commits (
    oid TEXT PRIMARY KEY,
    author_id INTEGER,
    committer_id INTEGER,
    tree_oid TEXT,
    timestamp INTEGER NOT NULL,
    message TEXT NOT NULL,
    FOREIGN KEY(author_id) REFERENCES authors(id),
    FOREIGN KEY(committer_id) REFERENCES authors(id)
);
```

## 6. Parents

```sql
CREATE TABLE commit_parents (
    commit_oid TEXT NOT NULL,
    parent_oid TEXT NOT NULL,
    parent_index INTEGER NOT NULL,
    PRIMARY KEY(commit_oid, parent_oid),
    FOREIGN KEY(commit_oid) REFERENCES commits(oid)
);
```

## 7. Files

```sql
CREATE TABLE files (
    id INTEGER PRIMARY KEY,
    path TEXT NOT NULL UNIQUE,
    first_commit_oid TEXT,
    last_commit_oid TEXT,
    language TEXT,
    is_current INTEGER NOT NULL DEFAULT 1
);
```

## 8. File changes

```sql
CREATE TABLE file_changes (
    id INTEGER PRIMARY KEY,
    commit_oid TEXT NOT NULL,
    file_id INTEGER NOT NULL,
    change_type TEXT NOT NULL,
    old_path TEXT,
    new_path TEXT,
    insertions INTEGER NOT NULL DEFAULT 0,
    deletions INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY(commit_oid) REFERENCES commits(oid),
    FOREIGN KEY(file_id) REFERENCES files(id)
);
```

## 9. Branches

```sql
CREATE TABLE branches (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    tip_oid TEXT,
    is_remote INTEGER NOT NULL DEFAULT 0,
    is_default INTEGER NOT NULL DEFAULT 0
);
```

## 10. Tags

```sql
CREATE TABLE tags (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    target_oid TEXT NOT NULL
);
```

## 11. Reflog

```sql
CREATE TABLE reflog_entries (
    id INTEGER PRIMARY KEY,
    reference TEXT NOT NULL,
    old_oid TEXT NOT NULL,
    new_oid TEXT NOT NULL,
    actor TEXT,
    timestamp INTEGER,
    message TEXT
);
```

## 12. Derived metrics

```sql
CREATE TABLE file_metrics (
    file_id INTEGER PRIMARY KEY,
    commit_count INTEGER NOT NULL,
    total_insertions INTEGER NOT NULL,
    total_deletions INTEGER NOT NULL,
    recent_churn INTEGER NOT NULL,
    contributor_count INTEGER NOT NULL,
    bug_fix_count INTEGER NOT NULL,
    complexity_score REAL,
    hotspot_score REAL,
    updated_at INTEGER NOT NULL
);
```

## 13. FTS

Use SQLite FTS5 for searchable textual fields.

Candidate indexed fields:

- commit message
- author
- file path
- branch name
- tag name
- symbol name when available

## 14. Index versioning

Schema migrations must be explicit.

Never rely on SQLite table shape inspection as the primary migration system.

Recommended:

```text
migrations/
001_initial.sql
002_add_reflog.sql
003_add_fts.sql
...
```

## 15. Cache invalidation

Derived metrics must record the source index version or snapshot.

When source data changes:

```text
source change
→ affected entities
→ invalidate derived records
→ recompute
```

## 16. Extended derived tables

The MVP schema above is sufficient for core analysis. Planned derived tables for later phases (V1+) should be added through migrations, not by altering existing tables:

```sql
-- ownership aggregates
CREATE TABLE file_ownership (
    file_id INTEGER NOT NULL,
    author_id INTEGER NOT NULL,
    weighted_contribution REAL NOT NULL,
    percentage REAL NOT NULL,
    PRIMARY KEY(file_id, author_id)
);

-- branch-to-commit membership
CREATE TABLE branch_commits (
    branch_id INTEGER NOT NULL,
    commit_oid TEXT NOT NULL,
    position INTEGER NOT NULL,
    PRIMARY KEY(branch_id, commit_oid)
);

-- rename lineage
CREATE TABLE file_renames (
    file_id INTEGER NOT NULL,
    commit_oid TEXT NOT NULL,
    old_path TEXT NOT NULL,
    new_path TEXT NOT NULL,
    detected_by TEXT NOT NULL  -- git | gitx_inference
);

-- language symbols (Tree-sitter era)
CREATE TABLE symbols (
    id INTEGER PRIMARY KEY,
    file_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    kind TEXT,
    line INTEGER
);

-- dependency tracking
CREATE TABLE dependencies (
    id INTEGER PRIMARY KEY,
    manifest_path TEXT NOT NULL,
    name TEXT NOT NULL,
    version TEXT,
    kind TEXT,           -- direct | indirect
    added_commit TEXT,
    removed_commit TEXT
);

CREATE TABLE dependency_events (
    id INTEGER PRIMARY KEY,
    dependency_id INTEGER NOT NULL,
    event_type TEXT NOT NULL,  -- added | removed | version_changed
    from_version TEXT,
    to_version TEXT,
    commit_oid TEXT
);
```

FTS search content (commit messages, authors, paths, symbols, branch/tag names) is covered by the FTS5 tables described in the FTS section; `search_index` naming and tokenizer settings should be finalized when the search engine lands.
