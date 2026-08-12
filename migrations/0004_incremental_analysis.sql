-- v4: incremental analysis cache support (docs/13 §3).
--
-- file_ownership gains an absolute per-author line count so a refresh can
-- apply the delta of new commits to ownership shares instead of recomputing
-- the whole analysis. Rows written before v4 keep a default of 0 and are only
-- read when an incremental update touches that file, where the missing
-- baseline is resolved by the next full scan.

ALTER TABLE file_ownership ADD COLUMN lines INTEGER NOT NULL DEFAULT 0;

UPDATE index_metadata SET value = '4' WHERE key = 'schema_version';
