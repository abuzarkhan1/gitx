-- GitX schema, version 3 (corrective: FTS5 sync triggers).
--
-- The runtime source of truth is the embedded constant SCHEMA_V3 in
-- crates/gitx-storage/src/migrations.rs. This file mirrors it exactly and
-- exists for review, tooling, and the documented repository layout
-- (docs/04-SYSTEM-ARCHITECTURE.md). Keep both in sync.
--
-- v3 fixes a bug in the v2 FTS5 sync triggers: the FTS5 `'delete'` special
-- INSERT command is only valid for contentless/external-content tables and
-- raised "SQL logic error" on our normal-content tables. The corrected
-- triggers use a plain `DELETE FROM <fts> WHERE rowid = ...`. Existing v2
-- databases keep the broken triggers (the version guard skips re-running
-- SCHEMA_V2), so this migration drops and recreates them.

DROP TRIGGER IF EXISTS commits_ad;
DROP TRIGGER IF EXISTS commits_au;
DROP TRIGGER IF EXISTS files_ad;
DROP TRIGGER IF EXISTS files_au;
DROP TRIGGER IF EXISTS authors_ad;
DROP TRIGGER IF EXISTS branches_ad;
DROP TRIGGER IF EXISTS tags_ad;

CREATE TRIGGER commits_ad AFTER DELETE ON commits BEGIN
    DELETE FROM commits_fts WHERE rowid = old.rowid;
END;
CREATE TRIGGER commits_au AFTER UPDATE OF message ON commits BEGIN
    DELETE FROM commits_fts WHERE rowid = old.rowid;
    INSERT INTO commits_fts(rowid, oid, message) VALUES (new.rowid, new.oid, new.message);
END;
CREATE TRIGGER files_ad AFTER DELETE ON files BEGIN
    DELETE FROM files_fts WHERE rowid = old.id;
END;
CREATE TRIGGER files_au AFTER UPDATE OF path ON files BEGIN
    DELETE FROM files_fts WHERE rowid = old.id;
    INSERT INTO files_fts(rowid, path) VALUES (new.id, new.path);
END;
CREATE TRIGGER authors_ad AFTER DELETE ON authors BEGIN
    DELETE FROM authors_fts WHERE rowid = old.id;
END;
CREATE TRIGGER branches_ad AFTER DELETE ON branches BEGIN
    DELETE FROM branches_fts WHERE rowid = old.id;
END;
CREATE TRIGGER tags_ad AFTER DELETE ON tags BEGIN
    DELETE FROM tags_fts WHERE rowid = old.id;
END;

UPDATE index_metadata SET value = '3' WHERE key = 'schema_version';
