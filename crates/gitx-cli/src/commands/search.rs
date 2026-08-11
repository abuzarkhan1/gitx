use crate::cli::Cli;
use crate::commands::index::build_index;
use crate::commands::{open_repo, print_json};
use gitx_search::{SearchFilters, SearchOrchestrator, SearchQuery, SqliteSearchBackend};
use serde_json::json;

#[allow(clippy::too_many_arguments)]
pub fn search(
    cli: &Cli,
    query: &str,
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
        code,
        history,
        since: None,
        author,
    };

    let search_query = SearchQuery::new(query).with_filters(filters);
    let backend = SqliteSearchBackend::new(&conn);
    let orchestrator = SearchOrchestrator::new(backend);
    let results = orchestrator.search(&search_query)?;

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
