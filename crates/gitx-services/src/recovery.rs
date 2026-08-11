//! `RecoveryService` (docs/04 §6): reflog inspection, unreachable-commit and
//! dangling-object discovery, and patch export (docs/12).

use gitx_analysis::recovery::{DanglingObject, UnreachableCommit};
use gitx_analysis::{RecoveryReport, collect_reflog};
use gitx_git::{ReflogEntry, Repository};

pub struct RecoveryService<'a> {
    pub repo: &'a Repository,
}

impl<'a> RecoveryService<'a> {
    pub fn new(repo: &'a Repository) -> Self {
        Self { repo }
    }

    /// Full recovery report: reflog + unreachable commits + dangling objects.
    pub fn analyze(&self) -> anyhow::Result<RecoveryReport> {
        Ok(gitx_analysis::analyze_recovery(self.repo)?)
    }

    pub fn reflog(&self) -> anyhow::Result<Vec<ReflogEntry>> {
        Ok(collect_reflog(self.repo)?)
    }

    pub fn unreachable(&self) -> anyhow::Result<Vec<UnreachableCommit>> {
        Ok(gitx_analysis::find_unreachable_commits(self.repo, None)?)
    }

    pub fn dangling(&self) -> anyhow::Result<Vec<DanglingObject>> {
        Ok(gitx_analysis::find_dangling_objects(self.repo, None, None)?)
    }

    /// Export a commit as a unified patch (docs/12 §6). Read-only.
    pub fn export_patch(&self, oid: &gitx_git::models::ObjectId) -> anyhow::Result<String> {
        Ok(gitx_git::diff::render_commit_patch(self.repo, *oid)?)
    }
}
