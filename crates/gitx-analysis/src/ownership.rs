use std::collections::HashMap;

/// Represents ownership data for an entity (like a file or module).
pub struct OwnershipData {
    /// Author ID mapped to lines owned or commits made.
    pub author_contributions: HashMap<i64, u32>,
}

impl OwnershipData {
    pub fn new() -> Self {
        Self {
            author_contributions: HashMap::new(),
        }
    }

    /// Total contributions across all authors.
    pub fn total_contributions(&self) -> u32 {
        self.author_contributions.values().sum()
    }

    /// Calculates ownership concentration (percentage of contributions by the top author).
    pub fn ownership_concentration(&self) -> f64 {
        let total = self.total_contributions() as f64;
        if total == 0.0 {
            return 0.0;
        }
        let max = self
            .author_contributions
            .values()
            .max()
            .copied()
            .unwrap_or(0) as f64;
        max / total
    }
}

impl Default for OwnershipData {
    fn default() -> Self {
        Self::new()
    }
}
