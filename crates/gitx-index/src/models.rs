#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Oid(pub String);

#[derive(Clone, Debug, PartialEq)]
pub struct Commit {
    pub id: Oid,
    pub parents: Vec<Oid>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RefInfo {
    pub name: String,
    pub target: Oid,
}
