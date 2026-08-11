use crate::result::SearchResult;
use std::cmp::Ordering;

/// Ranks the search results based on the following priorities:
/// 1. exact matches (simulated via score for now)
/// 2. path/name matches
/// 3. recent matches
/// 4. high-relevance textual matches
pub fn rank_results(results: &mut [SearchResult]) {
    results.sort_by(|a, b| {
        // High score first
        let score_a = a.score_if_applicable.unwrap_or(0.0);
        let score_b = b.score_if_applicable.unwrap_or(0.0);

        score_b.partial_cmp(&score_a).unwrap_or(Ordering::Equal)
    });
}
