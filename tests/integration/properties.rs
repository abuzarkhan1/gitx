//! Property-style checks (docs/14 §6): for a range of inputs the invariants
//! documented in docs/10 must hold — scores stay within 0–100, classification
//! bands partition the range, and ownership concentration is bounded.

use gitx_analysis::pipeline::analyze_repository;
use gitx_git::Repository;

#[path = "../common/mod.rs"]
mod common;
use common::FixtureRepo;

#[test]
fn scores_are_bounded_and_bands_partition_range() {
    let Some(repo) = FixtureRepo::new("prop-scores") else {
        eprintln!("skipping: git CLI unavailable");
        return;
    };
    // A history with many small commits so files span the metric range.
    for i in 0..12 {
        let file = format!("src/mod{i}.rs");
        repo.write(&file, &format!("pub fn f{i}() {{}}\npub fn g{i}() {{}}\n"));
        repo.commit(&format!("feat: add mod{i}"));
        if i % 3 == 0 {
            repo.write(
                &file,
                &format!("pub fn f{i}() {{}}\npub fn g{i}() {{}}\npub fn h{i}() {{}}\n"),
            );
            repo.commit(&format!("fix: extend mod{i}"));
        }
    }

    let gix = Repository::discover(repo.path()).expect("open fixture");
    let analysis = analyze_repository(&gix).expect("analyze");
    assert!(!analysis.files.is_empty(), "fixture must produce files");

    for file in &analysis.files {
        assert!(
            (0.0..=100.0).contains(&file.hotspot),
            "hotspot {} out of range: {}",
            file.path.display(),
            file.hotspot
        );
        assert!(
            (0.0..=100.0).contains(&file.risk),
            "risk {} out of range: {}",
            file.path.display(),
            file.risk
        );
        assert!(
            (0.0..=100.0).contains(&file.ownership_concentration),
            "ownership {} out of range: {}",
            file.path.display(),
            file.ownership_concentration
        );
        assert!(
            matches!(file.classification, "LOW" | "MEDIUM" | "HIGH" | "CRITICAL"),
            "unexpected band {}",
            file.classification
        );
    }

    // Health sub-scores and overall are normalized 0–100.
    let h = &analysis.health;
    for score in [
        h.overall_score,
        h.code_hotspots_score,
        h.ownership_risk_score,
        h.branch_hygiene_score,
        h.change_volatility_score,
        h.architecture_stability_score,
        h.recovery_risk_score,
    ] {
        assert!(
            (0.0..=100.0).contains(&score),
            "health sub-score out of range: {score}"
        );
    }
}

#[test]
fn many_files_keep_all_metrics_bounded() {
    let Some(repo) = FixtureRepo::new("prop-many") else {
        eprintln!("skipping: git CLI unavailable");
        return;
    };
    for i in 0..40 {
        repo.write(&format!("lib/a{i}.rs"), &format!("pub fn a{i}() {{}}\n"));
        repo.write(&format!("lib/b{i}.rs"), &format!("pub fn b{i}() {{}}\n"));
        repo.commit(&format!("feat: batch {i}"));
    }
    let gix = Repository::discover(repo.path()).expect("open fixture");
    let analysis = analyze_repository(&gix).expect("analyze");
    // 40 commits × 2 new files each.
    assert_eq!(analysis.files.len(), 80);
    for file in &analysis.files {
        assert!((0.0..=100.0).contains(&file.hotspot));
        assert!((0.0..=100.0).contains(&file.risk));
    }
}
