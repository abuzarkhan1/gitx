/// Evidence-backed hotspot score.
#[derive(Debug, Clone)]
pub struct HotspotScore {
    pub value: f64,
    pub classification: &'static str,
    pub time_window_days: u32,
}

/// Weights for the five hotspot signals. Defaults match the docs/10 formula;
/// configurable via `[analysis]` (docs/16 §3).
#[derive(Debug, Clone, Copy)]
pub struct HotspotWeights {
    pub change_frequency: f64,
    pub recent_churn: f64,
    pub bug_fix: f64,
    pub ownership: f64,
    pub complexity: f64,
}

impl Default for HotspotWeights {
    fn default() -> Self {
        Self {
            change_frequency: 0.25,
            recent_churn: 0.20,
            bug_fix: 0.20,
            ownership: 0.15,
            complexity: 0.20,
        }
    }
}

/// Calculate a hotspot score for a file with the default weights.
/// Based on change frequency, recent churn, bug-fix frequency, ownership concentration, and complexity.
/// All input metrics should be normalized to 0.0 - 100.0.
pub fn calculate_hotspot_score(
    normalized_change_frequency: f64,
    normalized_recent_churn: f64,
    normalized_bug_fix_frequency: f64,
    normalized_ownership_concentration: f64,
    normalized_complexity: f64,
    time_window_days: u32,
) -> HotspotScore {
    calculate_hotspot_score_with(
        HotspotWeights::default(),
        normalized_change_frequency,
        normalized_recent_churn,
        normalized_bug_fix_frequency,
        normalized_ownership_concentration,
        normalized_complexity,
        time_window_days,
    )
}

/// Like [`calculate_hotspot_score`], but with caller-provided weights.
#[allow(clippy::too_many_arguments)]
pub fn calculate_hotspot_score_with(
    weights: HotspotWeights,
    normalized_change_frequency: f64,
    normalized_recent_churn: f64,
    normalized_bug_fix_frequency: f64,
    normalized_ownership_concentration: f64,
    normalized_complexity: f64,
    time_window_days: u32,
) -> HotspotScore {
    let raw_score = weights.change_frequency * normalized_change_frequency
        + weights.recent_churn * normalized_recent_churn
        + weights.bug_fix * normalized_bug_fix_frequency
        + weights.ownership * normalized_ownership_concentration
        + weights.complexity * normalized_complexity;

    let classification = if raw_score > 80.0 {
        "CRITICAL"
    } else if raw_score > 60.0 {
        "HIGH"
    } else if raw_score > 30.0 {
        "MEDIUM"
    } else {
        "LOW"
    };

    HotspotScore {
        value: raw_score,
        classification,
        time_window_days,
    }
}

/// Composite risk score.
/// risk = hotspot_score + ownership_concentration + recent_churn + structural_complexity
pub fn calculate_risk_score(
    hotspot: &HotspotScore,
    ownership_concentration: f64,
    recent_churn: f64,
    structural_complexity: f64,
) -> f64 {
    hotspot.value + ownership_concentration + recent_churn + structural_complexity
}
