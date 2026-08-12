//! Index degraded states (docs/04 §9): `Indexed`, `PartiallyIndexed`,
//! `Failed`, `Unsupported`.

use gitx_git::Repository;
use serde::{Deserialize, Serialize};

/// The state of the persisted index relative to the repository.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IndexState {
    /// A valid index exists and matches HEAD (commits + analysis cache fresh).
    Indexed,
    /// An index exists but is stale (HEAD moved since the last scan, or the
    /// analysis cache is behind). Results still read the index when possible.
    PartiallyIndexed,
    /// An index exists but is corrupt or unreadable — a rebuild is required.
    Failed,
    /// No index at all — everything is computed live from Git.
    Unsupported,
}

/// Repository-level state: the Git state plus the index state.
#[derive(Debug, Clone, Serialize)]
pub struct State {
    pub git: String,
    pub index: IndexState,
}

impl State {
    /// Compute the current repository state deterministically.
    pub fn detect(repo: &Repository, index_path: &std::path::Path) -> State {
        let git = repo.state().unwrap_or_else(|| "clean".into());
        if !index_path.exists() {
            return State {
                git,
                index: IndexState::Unsupported,
            };
        }
        let conn = match rusqlite::Connection::open(index_path) {
            Ok(c) => c,
            Err(_) => {
                return State {
                    git,
                    index: IndexState::Failed,
                };
            }
        };
        // Corruption / newer-schema detection first.
        if gitx_storage::migrations::ensure_schema_compatible(&conn).is_err() {
            return State {
                git,
                index: IndexState::Failed,
            };
        }
        let commits: Option<i64> = conn
            .query_row("SELECT count(*) FROM commits", [], |row| row.get(0))
            .ok();
        let Some(commits) = commits else {
            return State {
                git,
                index: IndexState::Failed,
            };
        };
        if commits == 0 {
            return State {
                git,
                index: IndexState::PartiallyIndexed,
            };
        }
        // Fresh = stored last_head matches HEAD and the analysis cache is
        // current (docs/04 §9 Indexed vs PartiallyIndexed).
        let head = repo.head_commit_id().ok().map(|h| h.to_string());
        let last_head: Option<String> = conn
            .query_row(
                "SELECT value FROM index_metadata WHERE key = 'last_head'",
                [],
                |row| row.get(0),
            )
            .ok();
        let analysis_fresh = gitx_analysis::cache::is_fresh(&conn, repo);
        let head_matches = last_head.is_some() && last_head == head;
        // Indexed only when both the commit index and the analysis cache match
        // HEAD; anything else is partially indexed (stale) rather than failed.
        let index = if head_matches && analysis_fresh {
            IndexState::Indexed
        } else {
            IndexState::PartiallyIndexed
        };
        State { git, index }
    }
}
