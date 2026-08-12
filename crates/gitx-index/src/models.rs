#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Oid(pub String);

#[derive(Clone, Debug, PartialEq)]
pub struct Commit {
    pub id: Oid,
    pub parents: Vec<Oid>,
    /// Commit subject, when the provider can resolve it.
    pub message: Option<String>,
    /// Author timestamp (unix seconds), when the provider can resolve it.
    pub timestamp: Option<i64>,
    /// Author identity, when the provider can resolve it. Persisted so the
    /// index's `authors`/`commits.author_id` tables are populated by the
    /// incremental indexer exactly like `build_index` does (search, stats,
    /// and contributor views read them).
    pub author_name: Option<String>,
    pub author_email: Option<String>,
    pub committer_name: Option<String>,
    pub committer_email: Option<String>,
    /// Root tree oid, when the provider can resolve it.
    pub tree_id: Option<String>,
}

impl Commit {
    pub fn new(id: Oid, parents: Vec<Oid>) -> Self {
        Self {
            id,
            parents,
            message: None,
            timestamp: None,
            author_name: None,
            author_email: None,
            committer_name: None,
            committer_email: None,
            tree_id: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RefInfo {
    pub name: String,
    pub target: Oid,
}
