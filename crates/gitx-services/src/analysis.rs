//! `AnalysisService` (docs/04 §6): repository intelligence with an
//! index-backed cache (docs/13 §3) and live fallback.

use crate::repository::default_index_path;
use gitx_analysis::hotspots::HotspotWeights;
use gitx_analysis::{RepoAnalysis, RegressionReport};
use gitx_git::Repository;

pub struct AnalysisService<'a> {
    pub repo: &'a Repository,
}

impl<'a> AnalysisService<'a> {
    pub fn new(repo: &'a Repository) -> Self {
        Self { repo }
    }

    /// Analyze the repository. When `use_cache` is true and a fresh analysis
    /// cache exists in the index, results are read from SQLite (docs/13 §3);
    /// otherwise the pipeline computes live from Git. `weights` only apply to
    /// the live path.
    pub fn analyze(&self, use_cache: bool, weights: HotspotWeights) -> anyhow::Result<RepoAnalysis> {
        if use_cache
            && let Some(a) = self.analysis_from_cache()?
        {
            return Ok(a);
        }
        gitx_analysis::analyze_repository_with(self.repo, weights)
    }

    /// Read a previously persisted analysis from the index.
    pub fn analysis_from_cache(&self) -> anyhow::Result<Option<RepoAnalysis>> {
        let path = default_index_path(self.repo);
        if !path.exists() {
            return Ok(None);
        }
        let conn = rusqlite::Connection::open(&path)?;
        if gitx_storage::migrations::ensure_schema_compatible(&conn).is_err() {
            return Ok(None);
        }
        if !gitx_analysis::cache::is_fresh(&conn, self.repo) {
            return Ok(None);
        }
        gitx_analysis::cache::load(&conn)
    }

    /// Regression analysis (docs/10 §9).
    pub fn regressions(&self, max: Option<usize>) -> anyhow::Result<RegressionReport> {
        gitx_analysis::analyze_regressions(self.repo, max)
    }
}
