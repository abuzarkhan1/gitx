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

        // Refs whose target changed since the last scan/refresh — used only
        // for rewritten-history detection below. The walk itself is seeded
        // with *all* current tips; the provider's boundary-stop walk skips
        // tips that are already indexed, so unchanged refs cost nothing.
        let changed_refs: Vec<&RefInfo> = current_refs
            .iter()
            .filter(|r| {
                !old_refs
                    .iter()
                    .any(|o| o.name == r.name && o.target == r.target)
            })
            .collect();

        // Determine deleted refs
        let current_ref_map: std::collections::HashMap<_, _> = current_refs
            .iter()
            .map(|r| (r.name.clone(), r.target.clone()))
            .collect();
        let mut deleted_refs = Vec::new();
        for old_ref in old_refs {
            if !current_ref_map.contains_key(&old_ref.name) {
                deleted_refs.push(old_ref.name.clone());
            }
        }

        // Which commits exist in the index already, loaded once (docs/13 §3):
        // the provider uses it to stop the walk at indexed boundaries so a
        // refresh touches O(new commits) objects, not the whole history.
        let indexed_oids = self.storage.get_indexed_oids()?;
        let to_process: Vec<_> = current_refs.iter().map(|r| r.target.clone()).collect();
        let commit_iter = self.git.walk_commits(&to_process, &indexed_oids)?;
        let mut progress = Progress::default();
        let mut visited = HashSet::new();

        // Rewritten-history detection (docs/09 §5): if the last-seen HEAD is
        // no longer reachable from any current ref, history was rewritten
        // (force-push / interactive rebase) and the index holds stale commits.
        //
        // Because the walk stops at indexed boundaries, the old HEAD is not
        // visited on a plain forward move — it is the boundary. It is
        // reachable (history intact) when (a) no ref moved at all, (b) the
        // ref HEAD points at moved but a walked commit has the old HEAD as a
        // direct parent (new commits on top of it), or (c) only non-HEAD refs
        // moved. Any other move of the HEAD lineage — amend/force-push,
        // `reset --hard` backward, a rewritten re-join — leaves the old HEAD
        // unreachable and is flagged.
        let previous_head = self.storage.get_meta("last_head")?;
        let head_ref = self.git.head_ref_name()?;
        let head_lineage_moved = current_head_ref(&current_refs, head_ref.as_deref())
            .map(|selected| changed_refs.iter().any(|c| c.name == selected.name))
            .unwrap_or(true);
        let mut saw_previous_head = previous_head.is_none() || !head_lineage_moved;

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
            if let Some(prev) = &previous_head {
                // New commit(s) sitting directly on top of the old HEAD: the
                // plain forward-move case, history intact.
                if commit.parents.iter().any(|p| p.0 == *prev) {
                    saw_previous_head = true;
                }
            }
            if previous_head.as_deref() == Some(commit.id.0.as_str()) {
                saw_previous_head = true;
            }
            if tx.is_commit_indexed(&commit.id)? {
                continue; // Added concurrently while we walked (docs/09 §7)
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

/// The ref the "mainline" HEAD lineage is tracked on: the branch HEAD points
/// at (when symbolic), falling back to the first local branch ref, then the
/// first ref. Used for rewritten-history detection (docs/09 §5).
fn current_head_ref<'a>(refs: &'a [RefInfo], head_ref: Option<&str>) -> Option<&'a RefInfo> {
    refs.iter()
        .find(|r| Some(r.name.as_str()) == head_ref)
        .or_else(|| refs.iter().find(|r| r.name.starts_with("refs/heads/")))
        .or_else(|| refs.first())
}

/// The tip of the branch HEAD points at (when symbolic), falling back to the
/// first local branch ref. Used to record the last-seen HEAD oid for
/// rewritten-history detection (docs/09 §5).
fn current_head<'a>(refs: &'a [RefInfo], head_ref: Option<&str>) -> Option<&'a str> {
    current_head_ref(refs, head_ref).map(|r| r.target.0.as_str())
}
