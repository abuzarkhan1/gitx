use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Database error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("Migration error: {0}")]
    Migration(String),
    #[error("index schema is newer than this build (v{stored} > v{supported}) — upgrade gitx or delete the index and rescan")]
    SchemaNewer { stored: i64, supported: i64 },
}

pub type Result<T> = std::result::Result<T, Error>;
