#[derive(Debug, Clone, Default)]
pub struct SearchFilters {
    pub commits: bool,
    pub files: bool,
    pub authors: bool,
    pub branches: bool,
    pub tags: bool,
    pub renames: bool,
    pub code: bool,
    pub history: bool,
    /// Search symbols extracted from source (docs/11 §2).
    pub symbols: bool,
    /// Search directories containing matching paths (docs/11 §2).
    pub directories: bool,
    pub since: Option<String>,
    pub author: Option<String>,
}

impl SearchFilters {
    pub fn is_empty(&self) -> bool {
        !(self.commits
            || self.files
            || self.authors
            || self.branches
            || self.tags
            || self.renames
            || self.code
            || self.history
            || self.symbols
            || self.directories)
    }

    /// If no explicit types are requested, enable a default broad search
    pub fn with_defaults_if_empty(mut self) -> Self {
        if self.is_empty() {
            self.commits = true;
            self.files = true;
            self.branches = true;
            self.tags = true;
            self.authors = true;
            self.symbols = true;
            self.directories = true;
        }
        self
    }
}
