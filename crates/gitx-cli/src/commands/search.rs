use crate::cli::Cli;
use crate::commands::index::build_index;
use crate::commands::{open_repo, print_json};
use gitx_search::{
    EntityType, SearchFilters, SearchOrchestrator, SearchQuery, SearchResult, SqliteSearchBackend,
};
use serde_json::json;
use std::path::Path;

#[allow(clippy::too_many_arguments)]
pub fn search(
    cli: &Cli,
    query: &str,
    since: Option<&str>,
    commits: bool,
    files: bool,
    authors: bool,
    branches: bool,
    tags: bool,
    renames: bool,
    code: bool,
    history: bool,
    author: Option<String>,
) -> anyhow::Result<()> {
    let repo = open_repo(cli)?;

    // `--since` is a commit-level filter; normalize it to unix seconds so the
    // SQLite backend can compare directly (docs/11 §4).
    let since = since
        .map(|s| crate::commands::parse_ts(s).map(|t| t.to_string()))
        .transpose()?;

    // Lazily build an in-memory FTS5 index from the repository (docs/11).
    let mut conn = rusqlite::Connection::open_in_memory()?;
    build_index(&mut conn, &repo)?;

    let filters = SearchFilters {
        commits,
        files,
        authors,
        branches,
        tags,
        renames,
        code: false, // code content is searched below, against the worktree
        history,
        since,
        author,
    };

    let search_query = SearchQuery::new(query).with_filters(filters);
    let backend = SqliteSearchBackend::new(&conn);
    let orchestrator = SearchOrchestrator::new(backend);
    let mut results = orchestrator.search(&search_query)?;

    // `--code` searches file contents, bounded to the current working tree
    // (docs/11 §4–§5: never stream every historical blob).
    if code {
        results.extend(search_code(&repo, query)?);
    }

    if cli.json {
        return print_json(&json!(
            results
                .iter()
                .map(|r| json!({
                    "entity_type": format!("{:?}", r.entity_type).to_lowercase(),
                    "id": r.id,
                    "display_name": r.display_name,
                    "match_context": r.match_context,
                    "score": r.score_if_applicable,
                }))
                .collect::<Vec<_>>()
        ));
    }

    for result in &results {
        let kind = format!("{:?}", result.entity_type).to_lowercase();
        println!("[{kind:<7}] {}", result.display_name);
    }
    println!("{} result(s)", results.len());
    Ok(())
}

/// Substring search over the working tree, bounded: `.git` and binary files
/// are skipped, results are capped (docs/11 §5 code-content bounds).
fn search_code(repo: &gitx_git::Repository, query: &str) -> anyhow::Result<Vec<SearchResult>> {
    let mut results = Vec::new();
    let Some(work_dir) = repo.work_dir() else {
        return Ok(results);
    };
    if query.trim().is_empty() {
        return Ok(results);
    }

    // A tiny, dependency-free walker (ignores `.git` by construction; hidden
    // dirs are skipped to stay fast and bounded).
    fn walk(dir: &Path, root: &Path, query: &str, results: &mut Vec<SearchResult>, depth: usize) {
        if depth > 12 || results.len() >= 50 {
            return;
        }
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            if results.len() >= 50 {
                break;
            }
            let path = entry.path();
            let name = entry.file_name();
            let name = name.to_string_lossy();
            // Do not follow symlinks: a symlink could point outside the
            // repository root (docs/15 §5 symlink traversal defense).
            let file_type = entry
                .file_type()
                .ok()
                .or_else(|| std::fs::symlink_metadata(&path).map(|m| m.file_type()).ok());
            let Some(file_type) = file_type else { continue };
            if file_type.is_symlink() {
                continue;
            }
            if name == ".git" || (name.starts_with('.') && file_type.is_dir()) {
                continue;
            }
            if file_type.is_dir() {
                walk(&path, root, query, results, depth + 1);
                continue;
            }
            // Skip obvious binary extensions.
            let binary = path
                .extension()
                .map(|e| {
                    matches!(
                        e.to_string_lossy().as_ref(),
                        "png"
                            | "jpg"
                            | "jpeg"
                            | "gif"
                            | "ico"
                            | "pdf"
                            | "zip"
                            | "gz"
                            | "tar"
                            | "wasm"
                            | "woff"
                            | "ttf"
                            | "lock"
                            | "sqlite"
                            | "db"
                    )
                })
                .unwrap_or(false);
            if binary {
                continue;
            }
            let Ok(bytes) = std::fs::read(&path) else {
                continue;
            };
            // Cheap binary sniff: a NUL byte in the first 8 KiB.
            if bytes.iter().take(8192).any(|&b| b == 0) {
                continue;
            }
            let Ok(text) = String::from_utf8(bytes) else {
                continue;
            };
            let rel = path.strip_prefix(root).unwrap_or(&path);
            if let Some(line) = text.lines().position(|l| l.contains(query)) {
                let ctx = text
                    .lines()
                    .nth(line)
                    .map(|l| l.trim().chars().take(120).collect::<String>())
                    .unwrap_or_default();
                results.push(SearchResult {
                    entity_type: EntityType::Code,
                    id: rel.display().to_string(),
                    display_name: format!("{}:{line}", rel.display()),
                    match_context: Some(ctx),
                    score_if_applicable: None,
                });
            }
        }
    }

    walk(work_dir, work_dir, query, &mut results, 0);
    Ok(results)
}
