use crate::cli::Cli;
use crate::commands::index::build_index;
use crate::commands::{open_repo, print_json};
use gitx_search::{SearchFilters, SearchOrchestrator, SearchQuery, SqliteSearchBackend};
use serde_json::json;

#[allow(clippy::too_many_arguments)]
pub fn search(
    cli: &Cli,
    query: &str,
    since: Option<&str>,
    until: Option<&str>,
    path: Option<&str>,
    commits: bool,
    files: bool,
    authors: bool,
    branches: bool,
    tags: bool,
    renames: bool,
    code: bool,
    history: bool,
    symbols: bool,
    directories: bool,
    author: Option<String>,
) -> anyhow::Result<()> {
    let repo = open_repo(cli)?;
    // `[search] case_sensitive` (docs/16): FTS matching is case-insensitive,
    // so when case sensitivity is requested the hits are post-filtered.
    let case_sensitive = crate::commands::config::load_config_for(cli, &repo)?
        .search
        .case_sensitive;

    // `--since`/`--until` are commit-level filters; normalize to unix seconds
    // so the SQLite backend can compare directly (docs/11 §4).
    let since = since
        .map(|s| crate::commands::parse_ts(s).map(|t| t.to_string()))
        .transpose()?;
    let until = until
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
        symbols,
        directories,
        since,
        author,
        until,
        path: path.map(|p| p.to_string()),
    };

    let search_query = SearchQuery::new(query).with_filters(filters);
    let backend = SqliteSearchBackend::new(&conn);
    let orchestrator = SearchOrchestrator::new(backend);
    let mut results = orchestrator.search(&search_query)?;

    // `--code` searches file contents, bounded (docs/11 §4–§5: never stream
    // every historical blob): the working tree, falling back to the HEAD tree
    // for bare repositories (docs/11 §2). Results are capped at 50.
    let mut code_note: Option<String> = None;
    if code {
        let outcome = gitx_search::search_code_content(&repo, query, 50, case_sensitive);
        results.extend(outcome.results);
        code_note = Some(format!(
            "code content searched in the {} ({} file(s) scanned)",
            outcome.source, outcome.files_scanned
        ));
        if outcome.truncated {
            code_note = code_note.map(|n| format!("{n}; capped at 50 matches"));
        }
    }

    // Exact-case post-filter for the FTS-backed scopes (docs/16
    // `[search] case_sensitive`).
    if case_sensitive {
        results.retain(|r| {
            r.display_name.contains(query)
                || r.id.contains(query)
                || r.match_context.as_deref().unwrap_or("").contains(query)
        });
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
    if let Some(note) = code_note {
        // Bound note (docs/11 §5): code search never streams history, and a
        // cap is called out so the user knows results may be partial.
        println!("note: {note}");
    }
    Ok(())
}
