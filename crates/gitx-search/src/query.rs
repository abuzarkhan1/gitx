use crate::filter::SearchFilters;

#[derive(Debug, Clone)]
pub struct SearchQuery {
    pub term: String,
    pub filters: SearchFilters,
}

impl SearchQuery {
    pub fn new(term: impl Into<String>) -> Self {
        Self {
            term: term.into(),
            filters: SearchFilters::default(),
        }
    }

    pub fn with_filters(mut self, filters: SearchFilters) -> Self {
        self.filters = filters;
        self
    }
}
