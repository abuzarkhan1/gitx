//! `HistoryService` (docs/04 §6): commit timeline and file history.
//! The implementation lives in `gitx-history`; this re-exports it so the
//! service layer is the single entry point for the CLI and TUI.

pub use gitx_history::timeline::{HistoryService, TimelineOptions};
