use std::fmt;

/// Represents an Object ID in Git (e.g., SHA-1 or SHA-256)
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ObjectId(pub(crate) gix::ObjectId);

impl fmt::Display for ObjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ObjectId {
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    /// Parse a 40-character hex SHA-1 object id.
    pub fn from_hex(hex: &str) -> Option<Self> {
        gix::ObjectId::from_hex(hex.as_bytes()).ok().map(ObjectId)
    }
}

/// A representation of a Git Commit
#[derive(Clone, Debug)]
pub struct Commit {
    pub id: ObjectId,
    pub tree_id: ObjectId,
    pub parents: Vec<ObjectId>,
    pub author: Signature,
    pub committer: Signature,
    pub message: String,
}

/// A Git signature (author or committer)
#[derive(Clone, Debug)]
pub struct Signature {
    pub name: String,
    pub email: String,
    pub time: i64,   // seconds since epoch
    pub offset: i32, // timezone offset in seconds
}

/// A branch in the repository
#[derive(Clone, Debug)]
pub struct Branch {
    pub name: String,
    pub target: ObjectId,
    pub is_remote: bool,
}

/// A tag in the repository
#[derive(Clone, Debug)]
pub struct Tag {
    pub name: String,
    pub target: ObjectId,
}

/// A change in a single file between two trees
#[derive(Clone, Debug)]
pub struct FileChange {
    pub path: std::path::PathBuf,
    pub old_path: Option<std::path::PathBuf>,
    pub change_type: ChangeType,
    pub insertions: u32,
    pub deletions: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChangeType {
    Added,
    Modified,
    Deleted,
    Renamed,
    Copied,
    TypeChanged,
    Unknown,
}
