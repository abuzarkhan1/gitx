use crate::{EntityType, SearchBackend, SearchError, SearchQuery, SearchResult};
use rusqlite::Connection;

pub struct SqliteSearchBackend<'a> {
    pub conn: &'a Connection,
}

impl<'a> SqliteSearchBackend<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }
}

impl<'a> SearchBackend for SqliteSearchBackend<'a> {
    fn search_commits(&self, query: &SearchQuery) -> Result<Vec<SearchResult>, SearchError> {
        // The FTS table stores oid + message only; author/since filters need a
        // join to the normalized commits/authors tables (docs/11 §4).
        let mut sql = String::from(
            "SELECT c.oid, c.message, bm25(commits_fts) as score \
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
                let score: f64 = row.get(2)?;
                Ok(SearchResult {
                    entity_type: EntityType::Commit,
                    id,
                    display_name: msg.chars().take(50).collect(),
                    match_context: Some(msg),
                    score_if_applicable: Some(score),
                })
            })
            .map_err(|e| SearchError::Database(e.to_string()))?;

        let mut results = Vec::new();
        for r in rows {
            results.push(r.map_err(|e| SearchError::Database(e.to_string()))?);
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
                Ok(SearchResult {
                    entity_type: EntityType::File,
                    id: path.clone(),
                    display_name: path.clone(),
                    match_context: None,
                    score_if_applicable: Some(score),
                })
            })
            .map_err(|e| SearchError::Database(e.to_string()))?;

        let mut results = Vec::new();
        for r in rows {
            results.push(r.map_err(|e| SearchError::Database(e.to_string()))?);
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
                Ok(SearchResult {
                    entity_type: EntityType::Author,
                    id: email.clone(),
                    display_name: format!("{} <{}>", name, email),
                    match_context: None,
                    score_if_applicable: Some(score),
                })
            })
            .map_err(|e| SearchError::Database(e.to_string()))?;

        let mut results = Vec::new();
        for r in rows {
            results.push(r.map_err(|e| SearchError::Database(e.to_string()))?);
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
                Ok(SearchResult {
                    entity_type: EntityType::Branch,
                    id: name.clone(),
                    display_name: name.clone(),
                    match_context: None,
                    score_if_applicable: Some(score),
                })
            })
            .map_err(|e| SearchError::Database(e.to_string()))?;

        let mut results = Vec::new();
        for r in rows {
            results.push(r.map_err(|e| SearchError::Database(e.to_string()))?);
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
                Ok(SearchResult {
                    entity_type: EntityType::Tag,
                    id: name.clone(),
                    display_name: name.clone(),
                    match_context: None,
                    score_if_applicable: Some(score),
                })
            })
            .map_err(|e| SearchError::Database(e.to_string()))?;

        let mut results = Vec::new();
        for r in rows {
            results.push(r.map_err(|e| SearchError::Database(e.to_string()))?);
        }
        Ok(results)
    }
}
