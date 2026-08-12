//! `SearchService` (docs/04 §6): full-text search across commits, files,
//! authors, branches and tags (docs/11).

use crate::repository::default_index_path;
use gitx_git::Repository;

pub struct SearchService<'a> {
    pub repo: &'a Repository,
}

/// Search scope flags (docs/07 `gitx search`).
#[derive(Debug, Clone, Default)]
pub struct SearchOptions {
    pub since: Option<i64>,
    pub commits: bool,
    pub files: bool,
    pub authors: bool,
    pub branches: bool,
    pub tags: bool,
    pub renames: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SearchHit {
    pub scope: String,
    pub id: String,
    pub title: String,
    pub detail: String,
}

impl<'a> SearchService<'a> {
    pub fn new(repo: &'a Repository) -> Self {
        Self { repo }
    }

    /// Run an FTS query over the persisted index (building one in memory if
    /// no persisted index exists). Deterministic ordering (docs/11 §8).
    pub fn search(&self, query: &str, options: &SearchOptions) -> anyhow::Result<Vec<SearchHit>> {
        let path = default_index_path(self.repo);
        let conn = if path.exists() {
            rusqlite::Connection::open(&path)?
        } else {
            // No persisted index: build an in-memory one for the query.
            let mut conn = rusqlite::Connection::open_in_memory()?;
            gitx_storage::migrations::apply_migrations(&mut conn)?;
            crate::index::build_index(&mut conn, self.repo)?;
            conn
        };

        let mut hits = Vec::new();

        // Commits (message + author). `--since` filtering is applied by the
        // CLI layer before calling; the service matches across the whole index.
        if options.commits {
            let sql = "SELECT c.oid, c.message, COALESCE(a.name, '') FROM commits_fts f \
                       JOIN commits c ON c.rowid = f.rowid \
                       LEFT JOIN authors a ON a.id = c.author_id \
                       WHERE commits_fts MATCH ?1 \
                       ORDER BY c.timestamp DESC LIMIT 200";
            let mut stmt = conn.prepare(sql)?;
            let rows = stmt.query_map(rusqlite::params![query], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })?;
            for row in rows {
                let (oid, message, author) = row?;
                hits.push(SearchHit {
                    scope: "commit".into(),
                    id: oid,
                    title: message,
                    detail: author,
                });
            }
        }

        // Files (path).
        if options.files {
            let sql = "SELECT f.path FROM files_fts f WHERE files_fts MATCH ?1 LIMIT 200";
            let mut stmt = conn.prepare(sql)?;
            let rows = stmt.query_map([query], |row| row.get::<_, String>(0))?;
            for path in rows {
                let path = path?;
                hits.push(SearchHit {
                    scope: "file".into(),
                    id: path.clone(),
                    title: path,
                    detail: String::new(),
                });
            }
        }

        // Authors (name/email).
        if options.authors {
            let sql =
                "SELECT a.name, a.email FROM authors_fts a WHERE authors_fts MATCH ?1 LIMIT 100";
            let mut stmt = conn.prepare(sql)?;
            let rows = stmt.query_map([query], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?;
            for row in rows {
                let (name, email) = row?;
                hits.push(SearchHit {
                    scope: "author".into(),
                    id: email.clone(),
                    title: name,
                    detail: email,
                });
            }
        }

        // Branches.
        if options.branches {
            let sql = "SELECT name FROM branches_fts WHERE branches_fts MATCH ?1 LIMIT 100";
            let mut stmt = conn.prepare(sql)?;
            let rows = stmt.query_map([query], |row| row.get::<_, String>(0))?;
            for name in rows {
                let name = name?;
                hits.push(SearchHit {
                    scope: "branch".into(),
                    id: name.clone(),
                    title: name,
                    detail: String::new(),
                });
            }
        }

        // Tags.
        if options.tags {
            let sql = "SELECT name FROM tags_fts WHERE tags_fts MATCH ?1 LIMIT 100";
            let mut stmt = conn.prepare(sql)?;
            let rows = stmt.query_map([query], |row| row.get::<_, String>(0))?;
            for name in rows {
                let name = name?;
                hits.push(SearchHit {
                    scope: "tag".into(),
                    id: name.clone(),
                    title: name,
                    detail: String::new(),
                });
            }
        }

        Ok(hits)
    }
}
