pub mod branch;
pub mod classification;
pub mod health;
pub mod hotspots;
pub mod manifest;
pub mod metrics;
pub mod ownership;
pub mod pipeline;
pub mod recovery;
pub mod stats;

// Re-export common types
pub use branch::{BranchIntelligence, analyze_branch};
pub use classification::classify_commit_message;
pub use health::RepoHealth;
pub use hotspots::{
    HotspotScore, HotspotWeights, calculate_hotspot_score, calculate_hotspot_score_with,
    calculate_risk_score,
};
pub use metrics::FileMetrics;
pub use ownership::OwnershipData;
pub use pipeline::{FileAnalysis, RepoAnalysis, analyze_repository, analyze_repository_with};
pub use recovery::{RecoveryReport, analyze_recovery, collect_reflog, find_unreachable_commits};
pub use stats::{RepoStats, repository_stats};
