/// Repository health is a composite, deterministic overview derived from measurable signals.
#[derive(Debug, Clone)]
pub struct RepoHealth {
    pub overall_score: f64,
    pub code_hotspots_score: f64,
    pub ownership_risk_score: f64,
    pub branch_hygiene_score: f64,
    pub change_volatility_score: f64,
    pub architecture_stability_score: f64,
    pub recovery_risk_score: f64,
}

impl RepoHealth {
    /// Calculate the overall repository health score.
    /// Each sub-score should be normalized to 0-100.
    pub fn calculate(
        code_hotspots: f64,
        ownership_risk: f64,
        branch_hygiene: f64,
        change_volatility: f64,
        architecture_stability: f64,
        recovery_risk: f64,
    ) -> Self {
        // Weighted aggregation as specified in docs
        let overall_score = (code_hotspots * 0.25)
            + (ownership_risk * 0.20)
            + (branch_hygiene * 0.15)
            + (change_volatility * 0.15)
            + (architecture_stability * 0.15)
            + (recovery_risk * 0.10);

        Self {
            overall_score,
            code_hotspots_score: code_hotspots,
            ownership_risk_score: ownership_risk,
            branch_hygiene_score: branch_hygiene,
            change_volatility_score: change_volatility,
            architecture_stability_score: architecture_stability,
            recovery_risk_score: recovery_risk,
        }
    }
}

/// Health sub-score band labels (docs/10 §8): health scores are
/// higher-is-better, so the labels run POOR → EXCELLENT — the opposite
/// direction of the risk/hotspot bands. Shared by the CLI so the printed
/// bands can never drift from the TUI's color mapping again.
pub fn health_band(score: f64) -> &'static str {
    if score <= 30.0 {
        "POOR"
    } else if score < 61.0 {
        "FAIR"
    } else if score < 81.0 {
        "GOOD"
    } else {
        "EXCELLENT"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_band_partitions_0_100() {
        assert_eq!(health_band(0.0), "POOR");
        assert_eq!(health_band(29.9), "POOR");
        assert_eq!(health_band(30.0), "POOR");
        assert_eq!(health_band(60.9), "FAIR");
        assert_eq!(health_band(61.0), "GOOD");
        assert_eq!(health_band(80.9), "GOOD");
        assert_eq!(health_band(81.0), "EXCELLENT");
        assert_eq!(health_band(100.0), "EXCELLENT");
    }

    #[test]
    fn health_band_never_uses_risk_labels() {
        assert_ne!(health_band(95.0), "CRITICAL");
        assert_ne!(health_band(50.0), "HIGH");
    }
}
