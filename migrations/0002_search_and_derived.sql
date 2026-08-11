-- GitX schema, version 2 (FTS5 search index + derived intelligence tables).
--
-- The runtime source of truth is the embedded constant SCHEMA_V2 in
-- crates/gitx-storage/src/migrations.rs. This file mirrors it exactly and
-- exists for review, tooling, and the documented repository layout
-- (docs/04-SYSTEM-ARCHITECTURE.md). Keep both in sync.

-- Full-text search index (SQLite FTS5). These virtual tables back the
-- gitx-search crate; the search_index from docs/06 is realized here.
CREATE VIRTUAL TABLE commits_fts USING fts5(
    oid UNINDEXED,
    message,
    tokenize = 'porter unicode61'
);
CREATE VIRTUAL TABLE files_fts USING fts5(
    path,
    tokenize = 'porter unicode61'
);
CREATE VIRTUAL TABLE authors_fts USING fts5(
    name,
    email,
    tokenize = 'porter unicode61'
);
CREATE VIRTUAL TABLE branches_fts USING fts5(
    name,
    tokenize = 'porter unicode61'
);
CREATE VIRTUAL TABLE tags_fts USING fts5(
    name,
    tokenize = 'porter unicode61'
);

-- Keep the FTS index in sync with the underlying tables.
CREATE TRIGGER commits_ai AFTER INSERT ON commits BEGIN
    INSERT INTO commits_fts(rowid, oid, message) VALUES (new.rowid, new.oid, new.message);
END;
CREATE TRIGGER commits_ad AFTER DELETE ON commits BEGIN
    DELETE FROM commits_fts WHERE rowid = old.rowid;
END;
CREATE TRIGGER commits_au AFTER UPDATE OF message ON commits BEGIN
    DELETE FROM commits_fts WHERE rowid = old.rowid;
    INSERT INTO commits_fts(rowid, oid, message) VALUES (new.rowid, new.oid, new.message);
END;

CREATE TRIGGER files_ai AFTER INSERT ON files BEGIN
    INSERT INTO files_fts(rowid, path) VALUES (new.id, new.path);
END;
CREATE TRIGGER files_ad AFTER DELETE ON files BEGIN
    DELETE FROM files_fts WHERE rowid = old.id;
END;
CREATE TRIGGER files_au AFTER UPDATE OF path ON files BEGIN
    DELETE FROM files_fts WHERE rowid = old.id;
    INSERT INTO files_fts(rowid, path) VALUES (new.id, new.path);
END;

CREATE TRIGGER authors_ai AFTER INSERT ON authors BEGIN
    INSERT INTO authors_fts(rowid, name, email) VALUES (new.id, new.name, new.email);
END;
CREATE TRIGGER authors_ad AFTER DELETE ON authors BEGIN
    DELETE FROM authors_fts WHERE rowid = old.id;
END;

CREATE TRIGGER branches_ai AFTER INSERT ON branches BEGIN
    INSERT INTO branches_fts(rowid, name) VALUES (new.id, new.name);
END;
CREATE TRIGGER branches_ad AFTER DELETE ON branches BEGIN
    DELETE FROM branches_fts WHERE rowid = old.id;
END;

CREATE TRIGGER tags_ai AFTER INSERT ON tags BEGIN
    INSERT INTO tags_fts(rowid, name) VALUES (new.id, new.name);
END;
CREATE TRIGGER tags_ad AFTER DELETE ON tags BEGIN
    DELETE FROM tags_fts WHERE rowid = old.id;
END;

-- Derived tables (extended repository intelligence, docs/06).
CREATE TABLE file_renames (
    id INTEGER PRIMARY KEY,
    file_id INTEGER NOT NULL,
    commit_oid TEXT NOT NULL,
    old_path TEXT NOT NULL,
    new_path TEXT NOT NULL,
    FOREIGN KEY(file_id) REFERENCES files(id),
    FOREIGN KEY(commit_oid) REFERENCES commits(oid)
);

CREATE TABLE branch_commits (
    branch_id INTEGER NOT NULL,
    commit_oid TEXT NOT NULL,
    PRIMARY KEY(branch_id, commit_oid),
    FOREIGN KEY(branch_id) REFERENCES branches(id),
    FOREIGN KEY(commit_oid) REFERENCES commits(oid)
);

CREATE TABLE file_ownership (
    file_id INTEGER NOT NULL,
    author_id INTEGER NOT NULL,
    contribution_pct REAL NOT NULL DEFAULT 0,
    PRIMARY KEY(file_id, author_id),
    FOREIGN KEY(file_id) REFERENCES files(id),
    FOREIGN KEY(author_id) REFERENCES authors(id)
);

CREATE TABLE symbols (
    id INTEGER PRIMARY KEY,
    file_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    kind TEXT NOT NULL,
    line INTEGER,
    FOREIGN KEY(file_id) REFERENCES files(id)
);

CREATE TABLE dependencies (
    id INTEGER PRIMARY KEY,
    file_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    version TEXT,
    FOREIGN KEY(file_id) REFERENCES files(id)
);

CREATE TABLE dependency_events (
    id INTEGER PRIMARY KEY,
    commit_oid TEXT NOT NULL,
    dependency_name TEXT NOT NULL,
    event_type TEXT NOT NULL,
    version TEXT,
    FOREIGN KEY(commit_oid) REFERENCES commits(oid)
);

CREATE TABLE hotspots (
    file_id INTEGER PRIMARY KEY,
    hotspot_score REAL NOT NULL,
    risk_score REAL NOT NULL,
    classification TEXT NOT NULL,
    computed_at INTEGER NOT NULL,
    FOREIGN KEY(file_id) REFERENCES files(id)
);

CREATE TABLE metrics (
    id INTEGER PRIMARY KEY,
    scope TEXT NOT NULL,
    scope_id INTEGER,
    metric_key TEXT NOT NULL,
    metric_value REAL NOT NULL,
    computed_at INTEGER NOT NULL
);

CREATE INDEX idx_file_renames_file ON file_renames(file_id);
CREATE INDEX idx_branch_commits_commit ON branch_commits(commit_oid);
CREATE INDEX idx_file_ownership_file ON file_ownership(file_id);
CREATE INDEX idx_symbols_file ON symbols(file_id);
CREATE INDEX idx_dependencies_file ON dependencies(file_id);
CREATE INDEX idx_dependency_events_commit ON dependency_events(commit_oid);
CREATE INDEX idx_metrics_scope ON metrics(scope, scope_id);
UPDATE index_metadata SET value = '2' WHERE key = 'schema_version';
