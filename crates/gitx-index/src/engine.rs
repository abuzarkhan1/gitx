use crate::contracts::{GitProvider, StorageProvider};
use crate::error::IndexerError;
use crate::models::RefInfo;
use crate::progress::{Progress, ProgressReporter};
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};

const BATCH_SIZE: usize = 1000;

pub struct Indexer<'a> {
    git: &'a dyn GitProvider,
    storage: &'a dyn StorageProvider,
}

impl<'a> Indexer<'a> {
    pub fn new(git: &'a dyn GitProvider, storage: &'a dyn StorageProvider) -> Self {
        Self { git, storage }
    }

    /// Performs the initial complete scan of the repository.
    pub fn scan<P: ProgressReporter>(&self, reporter: &mut P) -> Result<(), IndexerError> {
        self.scan_with(reporter, &AtomicBool::new(false))
    }

    /// Like [`Indexer::scan`], but cancellable via `cancelled` (docs/09 §7).
    /// The open transaction is dropped (rolled back) on cancellation, leaving
    /// the index in its previous consistent state.
    pub fn scan_with<P: ProgressReporter>(
        &self,
        reporter: &mut P,
        cancelled: &AtomicBool,
    ) -> Result<(), IndexerError> {
        self.process_updates(&[], reporter, cancelled)
    }

    /// Performs an incremental refresh, detecting changed refs and fetching minimal required commits.
    pub fn refresh<P: ProgressReporter>(&self, reporter: &mut P) -> Result<(), IndexerError> {
        self.refresh_with(reporter, &AtomicBool::new(false))
    }

    /// Like [`Indexer::refresh`], but cancellable via `cancelled` (docs/09 §7).
    pub fn refresh_with<P: ProgressReporter>(
        &self,
        reporter: &mut P,
        cancelled: &AtomicBool,
    ) -> Result<(), IndexerError> {
        let old_refs = self.storage.get_indexed_refs()?;
        self.process_updates(&old_refs, reporter, cancelled)
    }

    fn process_updates<P: ProgressReporter>(
        &self,
        old_refs: &[RefInfo],
        reporter: &mut P,
        cancelled: &AtomicBool,
    ) -> Result<(), IndexerError> {
        let current_refs = self.git.read_refs()?;

        // Find which refs need updating
        let mut to_process = Vec::new();
        let mut current_ref_map = std::collections::HashMap::new();

        for r in &current_refs {
            current_ref_map.insert(r.name.clone(), r.target.clone());
            to_process.push(r.target.clone());
        }

        // Determine deleted refs
        let mut deleted_refs = Vec::new();
        for old_ref in old_refs {
            if !current_ref_map.contains_key(&old_ref.name) {
                deleted_refs.push(old_ref.name.clone());
            }
        }

        // Walk commits from current refs
        let commit_iter = self.git.walk_commits(&to_process)?;
        let mut progress = Progress::default();
        let mut visited = HashSet::new();

        // Rewritten-history detection (docs/09 §5): if the last-seen HEAD is
        // no longer reachable from any current ref, history was rewritten
        // (force-push / interactive rebase) and the index holds stale commits.
        let previous_head = self.storage.get_meta("last_head")?;
        let mut saw_previous_head = previous_head.is_none();

        let mut tx = self.storage.begin_transaction()?;
        let mut batch_count = 0;

        for commit_res in commit_iter {
            // Cancellation: roll the open transaction back (it is dropped
            // uncommitted below) and abort cleanly (docs/09 §7).
            if cancelled.load(Ordering::Relaxed) {
                return Err(IndexerError::Cancelled);
            }
            let commit = commit_res?;
            if visited.contains(&commit.id) {
                continue;
            }
            if previous_head.as_deref() == Some(commit.id.0.as_str()) {
                saw_previous_head = true;
            }
            if tx.is_commit_indexed(&commit.id)? {
                continue; // Stop going deeper on this branch if already indexed
            }

            visited.insert(commit.id.clone());
            tx.write_commit(&commit)?;
            batch_count += 1;

            if batch_count >= BATCH_SIZE {
                tx.commit()?;
                tx = self.storage.begin_transaction()?;
                batch_count = 0;
            }

            progress.commits_processed += 1;
            reporter.report(&progress);
        }

        // Record HEAD for the next refresh, and flag rewritten history so the
        // CLI can warn the user (the stale commits stay until a rebuild).
        let head_ref = self.git.head_ref_name()?;
        if let Some(head) = current_head(&current_refs, head_ref.as_deref()) {
            tx.write_meta("last_head", head)?;
        }
        tx.write_meta(
            "rewritten_detected",
            if previous_head.is_some() && !saw_previous_head {
                "1"
            } else {
                "0"
            },
        )?;

        for r in &current_refs {
            tx.write_ref(r)?;
        }

        for deleted_name in deleted_refs {
            tx.remove_ref(&deleted_name)?;
        }

        tx.commit()?;

        Ok(())
    }
}

/// The tip of the branch HEAD points at (when symbolic), falling back to the
/// first local branch ref. Used to track the true "mainline" HEAD for
/// rewritten-history detection (docs/09 §5).
fn current_head<'a>(refs: &'a [RefInfo], head_ref: Option<&str>) -> Option<&'a str> {
    refs.iter()
        .find(|r| Some(r.name.as_str()) == head_ref)
        .or_else(|| refs.iter().find(|r| r.name.starts_with("refs/heads/")))
        .or_else(|| refs.first())
        .map(|r| r.target.0.as_str())
}
