-- GitX schema, version 1 (initial).
--
-- The runtime source of truth is the embedded constant SCHEMA_V1 in
-- crates/gitx-storage/src/migrations.rs. This file mirrors it exactly and
-- exists for review, tooling, and the documented repository layout
-- (docs/04-SYSTEM-ARCHITECTURE.md). Keep both in sync.

CREATE TABLE index_metadata (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE repositories (
    id INTEGER PRIMARY KEY,
    root_path TEXT NOT NULL UNIQUE,
    git_dir TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE authors (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT,
    normalized_name TEXT,
    normalized_email TEXT
);

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

CREATE TABLE commit_parents (
    commit_oid TEXT NOT NULL,
    parent_oid TEXT NOT NULL,
    parent_index INTEGER NOT NULL,
    PRIMARY KEY(commit_oid, parent_oid),
    FOREIGN KEY(commit_oid) REFERENCES commits(oid)
);

CREATE TABLE files (
    id INTEGER PRIMARY KEY,
    path TEXT NOT NULL UNIQUE,
    first_commit_oid TEXT,
    last_commit_oid TEXT,
    language TEXT,
    is_current INTEGER NOT NULL DEFAULT 1
);

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

CREATE TABLE branches (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    tip_oid TEXT,
    is_remote INTEGER NOT NULL DEFAULT 0,
    is_default INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE tags (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    target_oid TEXT NOT NULL
);

CREATE TABLE reflog_entries (
    id INTEGER PRIMARY KEY,
    reference TEXT NOT NULL,
    old_oid TEXT NOT NULL,
    new_oid TEXT NOT NULL,
    actor TEXT,
    timestamp INTEGER,
    message TEXT
);

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

INSERT INTO index_metadata (key, value) VALUES ('schema_version', '1');
CREATE INDEX idx_commits_author ON commits(author_id);
CREATE INDEX idx_commits_committer ON commits(committer_id);
CREATE INDEX idx_commit_parents_parent ON commit_parents(parent_oid);
CREATE INDEX idx_file_changes_commit ON file_changes(commit_oid);
CREATE INDEX idx_file_changes_file ON file_changes(file_id);
