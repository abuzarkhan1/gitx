use crate::{EntityType, SearchBackend, SearchError, SearchQuery, SearchResult};
use rusqlite::Connection;

pub struct SqliteSearchBackend<'a> {
    pub conn: &'a Connection,
}

impl<'a> SqliteSearchBackend<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    fn hit(
        &self,
        entity_type: EntityType,
        id: String,
        display_name: String,
        match_context: Option<String>,
        score: f64,
        recent_ts: Option<i64>,
    ) -> SearchResult {
        SearchResult {
            entity_type,
            id,
            display_name,
            match_context,
            score_if_applicable: Some(score),
            recent_ts,
        }
    }
}

impl<'a> SearchBackend for SqliteSearchBackend<'a> {
    fn search_commits(&self, query: &SearchQuery) -> Result<Vec<SearchResult>, SearchError> {
        // The FTS table stores oid + message only; author/since filters need a
        // join to the normalized commits/authors tables (docs/11 §4).
        let mut sql = String::from(
            "SELECT c.oid, c.message, c.timestamp, bm25(commits_fts) as score \
             FROM commits_fts \
             JOIN commits c ON c.oid = commits_fts.oid \
             JOIN authors a ON a.id = c.author_id \
             WHERE commits_fts MATCH ?",
        );
        let mut params: Vec<String> = vec![query.term.clone()];

        if let Some(author) = &query.filters.author {
            sql.push_str(" AND (a.name LIKE ? OR a.email LIKE ?)");
            let pattern = format!("%{author}%");
            params.push(pattern.clone());
            params.push(pattern);
        }
        if let Some(since) = &query.filters.since {
            sql.push_str(" AND c.timestamp >= ?");
            params.push(since.clone());
        }
        if let Some(until) = &query.filters.until {
            sql.push_str(" AND c.timestamp <= ?");
            params.push(until.clone());
        }

        sql.push_str(" ORDER BY score LIMIT 50");

        let param_refs: Vec<&str> = params.iter().map(String::as_str).collect();
        let mut stmt = self
            .conn
            .prepare(&sql)
            .map_err(|e| SearchError::Database(e.to_string()))?;
        let rows = stmt
            .query_map(rusqlite::params_from_iter(param_refs), |row| {
                let id: String = row.get(0)?;
                let msg: String = row.get(1)?;
                let ts: i64 = row.get(2)?;
                let score: f64 = row.get(3)?;
                Ok((id, msg, ts, score))
            })
            .map_err(|e| SearchError::Database(e.to_string()))?;

        let mut results = Vec::new();
        for r in rows {
            let (id, msg, ts, score) = r.map_err(|e| SearchError::Database(e.to_string()))?;
            results.push(self.hit(
                EntityType::Commit,
                id.clone(),
                msg.chars().take(50).collect(),
                Some(msg),
                score,
                Some(ts),
            ));
        }
        Ok(results)
    }

    fn search_files(&self, query: &SearchQuery) -> Result<Vec<SearchResult>, SearchError> {
        let sql = r#"
            SELECT path, bm25(files_fts) as score
            FROM files_fts
            WHERE files_fts MATCH ?
            ORDER BY score
            LIMIT 50
        "#;
        let mut stmt = self
            .conn
            .prepare(sql)
            .map_err(|e| SearchError::Database(e.to_string()))?;
        let rows = stmt
            .query_map([&query.term], |row| {
                let path: String = row.get(0)?;
                let score: f64 = row.get(1)?;
                Ok((path, score))
            })
            .map_err(|e| SearchError::Database(e.to_string()))?;

        let mut results = Vec::new();
        for r in rows {
            let (path, score) = r.map_err(|e| SearchError::Database(e.to_string()))?;
            if let Some(prefix) = &query.filters.path {
                if !path.starts_with(prefix.as_str()) {
                    continue;
                }
            }
            results.push(self.hit(
                EntityType::File,
                path.clone(),
                path.clone(),
                None,
                score,
                None,
            ));
        }
        Ok(results)
    }

    fn search_authors(&self, query: &SearchQuery) -> Result<Vec<SearchResult>, SearchError> {
        let sql = r#"
            SELECT name, email, bm25(authors_fts) as score
            FROM authors_fts
            WHERE authors_fts MATCH ?
            ORDER BY score
            LIMIT 50
        "#;
        let mut stmt = self
            .conn
            .prepare(sql)
            .map_err(|e| SearchError::Database(e.to_string()))?;
        let rows = stmt
            .query_map([&query.term], |row| {
                let name: String = row.get(0)?;
                let email: String = row.get(1)?;
                let score: f64 = row.get(2)?;
                Ok((name, email, score))
            })
            .map_err(|e| SearchError::Database(e.to_string()))?;

        let mut results = Vec::new();
        for r in rows {
            let (name, email, score) = r.map_err(|e| SearchError::Database(e.to_string()))?;
            results.push(self.hit(
                EntityType::Author,
                email.clone(),
                format!("{name} <{email}>"),
                None,
                score,
                None,
            ));
        }
        Ok(results)
    }

    fn search_branches(&self, query: &SearchQuery) -> Result<Vec<SearchResult>, SearchError> {
        let sql = r#"
            SELECT name, bm25(branches_fts) as score
            FROM branches_fts
            WHERE branches_fts MATCH ?
            ORDER BY score
            LIMIT 50
        "#;
        let mut stmt = self
            .conn
            .prepare(sql)
            .map_err(|e| SearchError::Database(e.to_string()))?;
        let rows = stmt
            .query_map([&query.term], |row| {
                let name: String = row.get(0)?;
                let score: f64 = row.get(1)?;
                Ok((name, score))
            })
            .map_err(|e| SearchError::Database(e.to_string()))?;

        let mut results = Vec::new();
        for r in rows {
            let (name, score) = r.map_err(|e| SearchError::Database(e.to_string()))?;
            results.push(self.hit(
                EntityType::Branch,
                name.clone(),
                name.clone(),
                None,
                score,
                None,
            ));
        }
        Ok(results)
    }

    fn search_tags(&self, query: &SearchQuery) -> Result<Vec<SearchResult>, SearchError> {
        let sql = r#"
            SELECT name, bm25(tags_fts) as score
            FROM tags_fts
            WHERE tags_fts MATCH ?
            ORDER BY score
            LIMIT 50
        "#;
        let mut stmt = self
            .conn
            .prepare(sql)
            .map_err(|e| SearchError::Database(e.to_string()))?;
        let rows = stmt
            .query_map([&query.term], |row| {
                let name: String = row.get(0)?;
                let score: f64 = row.get(1)?;
                Ok((name, score))
            })
            .map_err(|e| SearchError::Database(e.to_string()))?;

        let mut results = Vec::new();
        for r in rows {
            let (name, score) = r.map_err(|e| SearchError::Database(e.to_string()))?;
            results.push(self.hit(
                EntityType::Tag,
                name.clone(),
                name.clone(),
                None,
                score,
                None,
            ));
        }
        Ok(results)
    }

    /// Rename history: rows in `file_renames` whose old or new path matches
    /// the term (docs/11 §4 `--renames`). Uses a LIKE match because paths are
    /// the whole entity here — FTS tokenization would split `src/foo.rs` and
    /// lose prefix matches.
    fn search_renames(&self, query: &SearchQuery) -> Result<Vec<SearchResult>, SearchError> {
        let pattern = format!("%{}%", query.term);
        let sql = r#"
            SELECT f.path, r.old_path, r.new_path
            FROM file_renames r
            JOIN files f ON f.id = r.file_id
            WHERE r.old_path LIKE ?1 OR r.new_path LIKE ?1
            LIMIT 50
        "#;
        let mut stmt = self
            .conn
            .prepare(sql)
            .map_err(|e| SearchError::Database(e.to_string()))?;
        let rows = stmt
            .query_map([&pattern], |row| {
                let path: String = row.get(0)?;
                let old: String = row.get(1)?;
                let new: String = row.get(2)?;
                Ok((path, old, new))
            })
            .map_err(|e| SearchError::Database(e.to_string()))?;

        let mut results = Vec::new();
        for r in rows {
            let (path, old, new) = r.map_err(|e| SearchError::Database(e.to_string()))?;
            results.push(self.hit(
                EntityType::Rename,
                new.clone(),
                format!("{old} → {new}"),
                Some(path),
                0.0,
                None,
            ));
        }
        Ok(results)
    }

    /// Symbols extracted from source (docs/11 §2). The `symbols` table is
    /// populated by `gitx scan`/`refresh`/`gitx symbols`; queried by name.
    fn search_symbols(&self, query: &SearchQuery) -> Result<Vec<SearchResult>, SearchError> {
        let pattern = format!("%{}%", query.term);
        let sql = r#"
            SELECT s.name, s.kind, s.line, f.path
            FROM symbols s
            JOIN files f ON f.id = s.file_id
            WHERE s.name LIKE ?1
            ORDER BY s.name
            LIMIT 50
        "#;
        let mut stmt = self
            .conn
            .prepare(sql)
            .map_err(|e| SearchError::Database(e.to_string()))?;
        let rows = stmt
            .query_map([&pattern], |row| {
                let name: String = row.get(0)?;
                let kind: String = row.get(1)?;
                let line: Option<i64> = row.get(2)?;
                let path: String = row.get(3)?;
                Ok((name, kind, line, path))
            })
            .map_err(|e| SearchError::Database(e.to_string()))?;

        let mut results = Vec::new();
        for r in rows {
            let (name, kind, line, path) = r.map_err(|e| SearchError::Database(e.to_string()))?;
            if let Some(prefix) = &query.filters.path {
                if !path.starts_with(prefix.as_str()) {
                    continue;
                }
            }
            let detail = line
                .map(|l| format!("{kind} · line {l} · {path}"))
                .unwrap_or_else(|| format!("{kind} · {path}"));
            results.push(self.hit(EntityType::Symbol, path, name, Some(detail), 0.0, None));
        }
        Ok(results)
    }

    /// Directories that contain files matching the term (docs/11 §2). Derived
    /// from the file paths: every path containing the term contributes its
    /// directory chain; matching directories are deduplicated.
    fn search_directories(&self, query: &SearchQuery) -> Result<Vec<SearchResult>, SearchError> {
        let pattern = format!("%{}%", query.term);
        let sql = "SELECT path FROM files WHERE path LIKE ?1 LIMIT 500";
        let mut stmt = self
            .conn
            .prepare(sql)
            .map_err(|e| SearchError::Database(e.to_string()))?;
        let rows = stmt
            .query_map([&pattern], |row| row.get::<_, String>(0))
            .map_err(|e| SearchError::Database(e.to_string()))?;

        let mut dirs: Vec<String> = Vec::new();
        for r in rows {
            let path = r.map_err(|e| SearchError::Database(e.to_string()))?;
            if let Some(prefix) = &query.filters.path {
                if !path.starts_with(prefix.as_str()) {
                    continue;
                }
            }
            let mut parent = std::path::Path::new(&path).parent();
            while let Some(p) = parent {
                let dir = p.display().to_string();
                if dir == "." || dir.is_empty() {
                    break;
                }
                if dir.to_lowercase().contains(&query.term.to_lowercase()) && !dirs.contains(&dir) {
                    dirs.push(dir.clone());
                }
                parent = p.parent();
            }
        }
        dirs.sort();
        Ok(dirs
            .into_iter()
            .take(50)
            .map(|d| self.hit(EntityType::Directory, d.clone(), d, None, 0.0, None))
            .collect())
    }
}
