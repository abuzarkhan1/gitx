use crate::query::SearchQuery;
use crate::ranking::rank_results;
use crate::result::SearchResult;
use crate::SearchError;

/// Trait representing a generic backend capable of executing the search logic
/// (e.g. against SQLite FTS5).
pub trait SearchBackend {
    fn search_commits(&self, query: &SearchQuery) -> Result<Vec<SearchResult>, SearchError>;
    fn search_files(&self, query: &SearchQuery) -> Result<Vec<SearchResult>, SearchError>;
    fn search_authors(&self, query: &SearchQuery) -> Result<Vec<SearchResult>, SearchError>;
    fn search_branches(&self, query: &SearchQuery) -> Result<Vec<SearchResult>, SearchError>;
    fn search_tags(&self, query: &SearchQuery) -> Result<Vec<SearchResult>, SearchError>;
}

/// Orchestrator to route queries to the correct backend and aggregate & rank results
pub struct SearchOrchestrator<B: SearchBackend> {
    backend: B,
}

impl<B: SearchBackend> SearchOrchestrator<B> {
    pub fn new(backend: B) -> Self {
        Self { backend }
    }

    pub fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>, SearchError> {
        let mut results = Vec::new();

        let filters = query.filters.clone().with_defaults_if_empty();

        if filters.commits || filters.history {
            results.extend(self.backend.search_commits(query)?);
        }

        if filters.files || filters.renames || filters.code {
            results.extend(self.backend.search_files(query)?);
        }

        if filters.authors {
            results.extend(self.backend.search_authors(query)?);
        }

        if filters.branches {
            results.extend(self.backend.search_branches(query)?);
        }

        if filters.tags {
            results.extend(self.backend.search_tags(query)?);
        }

        rank_results(&mut results);

        Ok(results)
    }
}
