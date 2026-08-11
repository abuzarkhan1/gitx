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
        let mut sql = String::from("SELECT oid, message, bm25(commits_fts) as score FROM commits_fts WHERE commits_fts MATCH ?");

        // Handle basic filters (e.g., author) in an FTS context if possible,
        // or assume an underlying view/table join. For simplicity, we append them if needed.
        if let Some(author) = &query.filters.author {
            // FTS5 boolean query syntax could be used, or a regular WHERE clause
            // if author is an indexed/unindexed column.
            sql.push_str(&format!(" AND author = '{}'", author.replace('\'', "''")));
        }

        sql.push_str(" ORDER BY score LIMIT 50");

        let mut stmt = self
            .conn
            .prepare(&sql)
            .map_err(|e| SearchError::Database(e.to_string()))?;
        let rows = stmt
            .query_map([&query.term], |row| {
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
