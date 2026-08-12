use crate::result::{EntityType, SearchResult};

/// The deterministic entity ordering used as the final tiebreak so results
/// are stable across runs and backends.
fn entity_order(e: &EntityType) -> u8 {
    match e {
        EntityType::Commit => 0,
        EntityType::File => 1,
        EntityType::Directory => 2,
        EntityType::Author => 3,
        EntityType::Branch => 4,
        EntityType::Tag => 5,
        EntityType::Symbol => 6,
        EntityType::Rename => 7,
        EntityType::Code => 8,
    }
}

/// Rank the search results by the documented priority tiers (docs/11 §8),
/// deterministically:
///
/// 1. **exact matches** — the id/name equals the term
/// 2. **path/name matches** — the id/name contains the term
/// 3. **recent matches** — the underlying entity changed within 30 days
/// 4. **high-relevance textual matches** — FTS bm25 score
///
/// Within a tier the bm25 score breaks ties; final ties are broken by the
/// entity type order and then the id, so ordering never depends on insertion
/// order or on SQLite internals.
pub fn rank_results(results: &mut [SearchResult], term: &str) {
    let term = term.trim().to_lowercase();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let cutoff = now - 30 * 86_400;

    results.sort_by(|a, b| {
        let ka = tier(a, &term, cutoff);
        let kb = tier(b, &term, cutoff);
        ka.cmp(&kb)
    });
}

/// Sort key for one result: (tier, reverse bm25, entity order, id).
fn tier(r: &SearchResult, term: &str, cutoff: i64) -> (u8, std::cmp::Reverse<i64>, u8, String) {
    let hay = format!("{} {}", r.id.to_lowercase(), r.display_name.to_lowercase());
    let exact = r.id.eq_ignore_ascii_case(term) || r.display_name.eq_ignore_ascii_case(term);
    let name_match = !term.is_empty() && hay.contains(term);
    let tier = if exact {
        0
    } else if name_match {
        1
    } else if r.recent_ts.is_some_and(|t| t >= cutoff) {
        2
    } else {
        3
    };
    let score = (r.score_if_applicable.unwrap_or(0.0) * 100.0) as i64;
    (
        tier,
        std::cmp::Reverse(score),
        entity_order(&r.entity_type),
        r.id.clone(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn res(entity: EntityType, id: &str, score: f64, recent: Option<i64>) -> SearchResult {
        SearchResult {
            entity_type: entity,
            id: id.to_string(),
            display_name: id.to_string(),
            match_context: None,
            score_if_applicable: Some(score),
            recent_ts: recent,
        }
    }

    #[test]
    fn exact_beats_name_match_beats_text() {
        let mut results = vec![
            res(EntityType::Commit, "a1b2c3", 10.0, None), // textual (no name match)
            res(EntityType::File, "lib.rs", 5.0, None),    // exact
            res(EntityType::File, "lib.rs.bak", 9.0, None), // name match
        ];
        rank_results(&mut results, "lib.rs");
        assert_eq!(results[0].id, "lib.rs");
        assert_eq!(results[1].id, "lib.rs.bak");
        assert_eq!(results[2].id, "a1b2c3");
    }

    #[test]
    fn recent_beats_textual_but_not_name() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let mut results = vec![
            res(EntityType::Commit, "old", 50.0, Some(now - 90 * 86_400)), // old, high bm25
            res(EntityType::Commit, "new", 10.0, Some(now - 2 * 86_400)),  // recent, low bm25
        ];
        rank_results(&mut results, "workspace");
        assert_eq!(results[0].id, "new");

        // But a name match still beats a recent text match.
        let mut results = vec![
            res(
                EntityType::File,
                "workspace.rs",
                1.0,
                Some(now - 400 * 86_400),
            ),
            res(EntityType::Commit, "new", 99.0, Some(now - 86_400)),
        ];
        rank_results(&mut results, "workspace");
        assert_eq!(results[0].id, "workspace.rs");
    }

    #[test]
    fn deterministic_tiebreak() {
        let mut a = vec![
            res(EntityType::Tag, "b", 5.0, None),
            res(EntityType::Tag, "a", 5.0, None),
        ];
        let mut b = a.clone();
        rank_results(&mut a, "x");
        rank_results(&mut b, "x");
        assert_eq!(
            a.iter().map(|r| r.id.clone()).collect::<Vec<_>>(),
            b.iter().map(|r| r.id.clone()).collect::<Vec<_>>()
        );
    }
}
