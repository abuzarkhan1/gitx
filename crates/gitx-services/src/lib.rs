//! Application services (docs/04 §6): the CLI and TUI invoke these services
//! rather than implementing business logic themselves.
//!
//! Each service is a thin, typed facade over the gitx-* crates, with
//! per-operation errors and a degraded-state model (docs/04 §9):
//!
//! ```text
//! Indexed           — a valid index exists and matches HEAD
//! PartiallyIndexed  — an index exists but is stale (HEAD moved, or the
//!                     analysis cache is missing/behind)
//! Failed            — an index exists but is corrupt/unreadable
//! Unsupported       — no index at all (analysis computed live)
//! ```
//!
//! Unsupported-language or per-file analysis failures never destroy the
//! repository index; they degrade individual results, not the whole scan.

pub mod analysis;
pub mod history;
pub mod index;
pub mod recovery;
pub mod repository;
pub mod search;
pub mod state;

pub use analysis::AnalysisService;
pub use history::HistoryService;
pub use index::IndexService;
pub use recovery::RecoveryService;
pub use repository::RepositoryService;
pub use search::{SearchHit, SearchOptions, SearchService};
pub use state::{IndexState, State};
