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
}

impl Commit {
    pub fn new(id: Oid, parents: Vec<Oid>) -> Self {
        Self {
            id,
            parents,
            message: None,
            timestamp: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RefInfo {
    pub name: String,
    pub target: Oid,
}
