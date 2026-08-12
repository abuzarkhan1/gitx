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
    /// Symbols extracted from source (docs/11 §2).
    pub symbols: bool,
    /// Directories containing matching paths (docs/11 §2).
    pub directories: bool,
    /// Code content, bounded to the working tree with a HEAD-tree fallback
    /// (docs/11 §4–§5: never stream every historical blob).
    pub code: bool,
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
    /// `raw` is the un-escaped user term, used by the code-content scope
    /// (which does plain substring matching, not FTS syntax).
    pub fn search(
        &self,
        query: &str,
        raw: &str,
        options: &SearchOptions,
    ) -> anyhow::Result<Vec<SearchHit>> {
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

        // Rename history (docs/11 §4 `--renames`): old → new paths.
        if options.renames {
            let pattern = format!("%{query}%");
            let sql = "SELECT r.old_path, r.new_path FROM file_renames r \
                       WHERE r.old_path LIKE ?1 OR r.new_path LIKE ?1 LIMIT 100";
            let mut stmt = conn.prepare(sql)?;
            let rows = stmt.query_map([pattern], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?;
            for row in rows {
                let (old_path, new_path) = row?;
                hits.push(SearchHit {
                    scope: "rename".into(),
                    id: new_path.clone(),
                    title: format!("{old_path} → {new_path}"),
                    detail: String::new(),
                });
            }
        }

        // Symbols (docs/11 §2): the symbols table is populated by scan/refresh.
        if options.symbols {
            let pattern = format!("%{query}%");
            let sql = "SELECT s.name, s.kind, s.line, f.path FROM symbols s \
                       JOIN files f ON f.id = s.file_id \
                       WHERE s.name LIKE ?1 ORDER BY s.name LIMIT 100";
            let mut stmt = conn.prepare(sql)?;
            let rows = stmt.query_map([pattern], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<i64>>(2)?,
                    row.get::<_, String>(3)?,
                ))
            })?;
            for row in rows {
                let (name, kind, line, path) = row?;
                hits.push(SearchHit {
                    scope: "symbol".into(),
                    id: path.clone(),
                    title: name,
                    detail: line
                        .map(|l| format!("{kind} · line {l} · {path}"))
                        .unwrap_or_else(|| format!("{kind} · {path}")),
                });
            }
        }

        // Directories (docs/11 §2): derived from file paths containing the term.
        if options.directories {
            let pattern = format!("%{query}%");
            let sql = "SELECT path FROM files WHERE path LIKE ?1 LIMIT 500";
            let mut stmt = conn.prepare(sql)?;
            let rows = stmt.query_map([pattern], |row| row.get::<_, String>(0))?;
            let mut dirs: Vec<String> = Vec::new();
            for path in rows {
                let path = path?;
                let mut parent = std::path::Path::new(&path).parent();
                while let Some(p) = parent {
                    let dir = p.display().to_string();
                    if dir == "." || dir.is_empty() {
                        break;
                    }
                    if dir.to_lowercase().contains(&query.to_lowercase()) && !dirs.contains(&dir) {
                        dirs.push(dir.clone());
                    }
                    parent = p.parent();
                }
            }
            dirs.sort();
            for dir in dirs.into_iter().take(50) {
                hits.push(SearchHit {
                    scope: "directory".into(),
                    id: dir.clone(),
                    title: dir,
                    detail: String::new(),
                });
            }
        }

        // Code content (docs/11 §2, §4): bounded substring search over the
        // working tree, with a HEAD-tree fallback for bare repositories. The
        // cap (50) keeps it fast and honest; the raw term is used (not the
        // FTS-escaped query) because this is plain substring matching.
        if options.code && !raw.trim().is_empty() {
            let outcome = gitx_search::search_code_content(self.repo, raw.trim(), 50);
            for r in outcome.results {
                hits.push(SearchHit {
                    scope: "code".into(),
                    id: r.id.clone(),
                    title: r.display_name,
                    detail: r.match_context.unwrap_or_default(),
                });
            }
        }

        Ok(hits)
    }
}
