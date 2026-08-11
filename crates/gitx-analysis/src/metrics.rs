use chrono::{DateTime, Utc};

/// Basic file-level metrics calculated from history.
#[derive(Debug, Clone, Default)]
pub struct FileMetrics {
    /// Number of times the file was changed
    pub change_frequency: u32,
    /// Total lines added to the file
    pub lines_added: u32,
    /// Total lines deleted from the file
    pub lines_deleted: u32,
    /// First time the file was introduced
    pub first_introduced: Option<DateTime<Utc>>,
    /// Last time the file was modified
    pub last_modified: Option<DateTime<Utc>>,
    /// Number of bug-fix classified commits touching this file
    pub bug_fix_count: u32,
    /// Number of unique authors who touched this file
    pub unique_contributors: u32,
}

impl FileMetrics {
    /// Calculates churn (lines added + deleted).
    pub fn churn(&self) -> u32 {
        self.lines_added + self.lines_deleted
    }

    /// Calculates a recency score (1.0 = today, decreasing linearly to 0.0 over 1 year).
    pub fn recency_score(&self, now: DateTime<Utc>) -> f64 {
        if let Some(last) = self.last_modified {
            let days = (now - last).num_days() as f64;
            if days <= 0.0 {
                return 1.0;
            }
            let score = 1.0 - (days / 365.0);
            score.clamp(0.0, 1.0)
        } else {
            0.0
        }
    }
}
