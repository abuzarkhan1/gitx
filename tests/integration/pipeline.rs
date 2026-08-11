//! Verifies the rayon-parallel analysis walk (docs/13 §6) is deterministic:
//! two runs over the same repository must produce identical results even
//! though commit diffs are computed in parallel on a bounded worker pool.

use gitx_analysis::pipeline::analyze_repository;
use gitx_git::Repository;

#[path = "../common/mod.rs"]
mod common;
use common::FixtureRepo;

#[test]
fn parallel_analysis_is_deterministic() {
    let Some(repo) = FixtureRepo::new("pipeline") else {
        eprintln!("skipping: git CLI unavailable");
        return;
    };
    // A small multi-file history with a fix commit and a rename.
    repo.write("src/lib.rs", "pub fn a() {}\npub fn b() {}\n");
    repo.commit("feat: initial library");
    repo.write(
        "src/lib.rs",
        "pub fn a() {}\npub fn b() {}\npub fn c() {}\n",
    );
    repo.commit("feat: add c");
    repo.write("README.md", "# Fixture\n");
    repo.commit("docs: readme");
    repo.write("src/lib.rs", "pub fn a() {}\n");
    repo.commit("fix: remove broken c");

    let gix = Repository::discover(repo.path()).expect("open fixture");

    let first = analyze_repository(&gix).expect("first analysis");
    let second = analyze_repository(&gix).expect("second analysis");

    assert_eq!(first.total_commits, second.total_commits);
    assert_eq!(first.total_contributors, second.total_contributors);
    assert_eq!(first.current_files, second.current_files);
    assert_eq!(first.files.len(), second.files.len());

    for (a, b) in first.files.iter().zip(second.files.iter()) {
        assert_eq!(a.path, b.path, "file order differs between runs");
        assert_eq!(
            a.metrics.change_frequency,
            b.metrics.change_frequency,
            "{}",
            a.path.display()
        );
        assert_eq!(
            a.metrics.lines_added,
            b.metrics.lines_added,
            "{}",
            a.path.display()
        );
        assert_eq!(
            a.metrics.bug_fix_count,
            b.metrics.bug_fix_count,
            "{}",
            a.path.display()
        );
        assert_eq!(
            a.ownership_concentration,
            b.ownership_concentration,
            "{}",
            a.path.display()
        );
        assert_eq!(a.hotspot, b.hotspot, "{}", a.path.display());
        assert_eq!(a.risk, b.risk, "{}", a.path.display());
        assert_eq!(a.classification, b.classification, "{}", a.path.display());
    }

    // Health sub-scores must be stable too.
    assert_eq!(
        first.health.code_hotspots_score,
        second.health.code_hotspots_score
    );
    assert_eq!(
        first.health.ownership_risk_score,
        second.health.ownership_risk_score
    );
    assert_eq!(
        first.health.branch_hygiene_score,
        second.health.branch_hygiene_score
    );
    assert_eq!(
        first.health.change_volatility_score,
        second.health.change_volatility_score
    );
    assert_eq!(
        first.health.architecture_stability_score,
        second.health.architecture_stability_score
    );
}
