use serde::{Deserialize, Serialize};

/// Type of change made to a file in a commit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeType {
    Added,
    Modified,
    Deleted,
    Renamed,
    Copied,
    TypeChanged,
    Unknown,
}

/// Heuristic classification of a commit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommitClassification {
    Feature,
    Fix,
    Refactor,
    Docs,
    Test,
    Chore,
    Revert,
    Merge,
    Unknown,
}
