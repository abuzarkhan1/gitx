use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EntityType {
    Commit,
    File,
    Branch,
    Tag,
    Author,
    Symbol,
    /// A directory that contains files matching the query (docs/11 §2).
    Directory,
    /// A rename event (old → new path, docs/11 §4 `--renames`).
    Rename,
    /// A match inside file contents (`gitx search --code`, docs/11 §4).
    Code,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub entity_type: EntityType,
    pub id: String,
    pub display_name: String,
    pub match_context: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score_if_applicable: Option<f64>,
    /// Recency signal for the ranking tiers (docs/11 §8): the timestamp of
    /// the underlying entity (commit author time, file last change...).
    /// Absent when unknown; used to lift recent matches above older text
    /// matches without ever reordering exact/name matches.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recent_ts: Option<i64>,
}
