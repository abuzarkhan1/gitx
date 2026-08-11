use crate::contracts::{GitProvider, StorageProvider};
use crate::error::IndexerError;
use crate::models::RefInfo;
use crate::progress::{Progress, ProgressReporter};
use std::collections::HashSet;

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
        self.process_updates(&[], reporter)
    }

    /// Performs an incremental refresh, detecting changed refs and fetching minimal required commits.
    pub fn refresh<P: ProgressReporter>(&self, reporter: &mut P) -> Result<(), IndexerError> {
        let old_refs = self.storage.get_indexed_refs()?;
        self.process_updates(&old_refs, reporter)
    }

    fn process_updates<P: ProgressReporter>(
        &self,
        old_refs: &[RefInfo],
        reporter: &mut P,
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

        let mut tx = self.storage.begin_transaction()?;
        let mut batch_count = 0;

        for commit_res in commit_iter {
            let commit = commit_res?;
            if visited.contains(&commit.id) {
                continue;
            }
            if self.storage.is_commit_indexed(&commit.id)? {
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
