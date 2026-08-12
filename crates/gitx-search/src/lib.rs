pub mod code;
pub mod filter;
pub mod orchestrator;
pub mod query;
pub mod ranking;
pub mod result;

pub use code::*;
pub use filter::*;
pub use orchestrator::*;
pub use query::*;
pub use ranking::*;
pub use result::*;

#[derive(Debug, thiserror::Error)]
pub enum SearchError {
    #[error("Database error: {0}")]
    Database(String),
    #[error("Query parse error: {0}")]
    QueryParse(String),
    #[error("Internal error: {0}")]
    Internal(String),
}
pub mod sqlite;
pub use sqlite::*;
