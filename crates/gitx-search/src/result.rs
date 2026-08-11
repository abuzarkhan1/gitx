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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub entity_type: EntityType,
    pub id: String,
    pub display_name: String,
    pub match_context: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score_if_applicable: Option<f64>,
}
