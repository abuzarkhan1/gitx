use gitx_analysis::{
    analyze_recovery, analyze_repository, find_unreachable_commits, repository_stats,
};
use gitx_git::Repository;

#[path = "../common/mod.rs"]
mod common;
use common::sample_repo;

#[test]
fn stats_are_deterministic() {
    let Some(repo) = sample_repo() else {
        eprintln!("skipping: git CLI unavailable");
        return;
    };
    let gix = Repository::discover(repo.path()).expect("open fixture");

    let stats = repository_stats(&gix).expect("stats");
    assert_eq!(stats.commits, 3);
    assert_eq!(stats.contributors, 1);
    assert_eq!(stats.files, 3); // main.rs, src/lib.rs, README.md
    assert_eq!(stats.branches, 1);
    assert_eq!(stats.tags, 0);
    assert!(stats.languages.iter().any(|(ext, _)| ext == "rs"));
}

#[test]
fn hotspots_are_sorted_and_bounded() {
    let Some(repo) = sample_repo() else {
        eprintln!("skipping: git CLI unavailable");
        return;
    };
    let gix = Repository::discover(repo.path()).expect("open fixture");

    let analysis = analyze_repository(&gix).expect("analysis");
    assert_eq!(analysis.total_commits, 3);
    assert!(!analysis.files.is_empty());

    for window in analysis.files.windows(2) {
        assert!(
            window[0].hotspot >= window[1].hotspot,
            "hotspots must be sorted descending"
        );
    }
    for file in &analysis.files {
        assert!((0.0..=100.0).contains(&file.hotspot));
        assert!((0.0..=100.0).contains(&file.risk));
        assert!((0.0..=100.0).contains(&file.ownership_concentration));
        assert!(matches!(
            file.classification,
            "LOW" | "MEDIUM" | "HIGH" | "CRITICAL"
        ));
    }
}

#[test]
fn health_sub_scores_are_bounded_and_evidence_backed() {
    let Some(repo) = sample_repo() else {
        eprintln!("skipping: git CLI unavailable");
        return;
    };
    let gix = Repository::discover(repo.path()).expect("open fixture");

    let analysis = analyze_repository(&gix).expect("analysis");
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
            "health score out of range: {score}"
        );
    }
}

#[test]
fn reflog_is_readable_and_unreachable_detects_deleted_branch() {
    let Some(repo) = sample_repo() else {
        eprintln!("skipping: git CLI unavailable");
        return;
    };
    let gix = Repository::discover(repo.path()).expect("open fixture");

    // Reflogs exist (git creates them by default in non-bare repos).
    let report = analyze_recovery(&gix).expect("recovery report");
    assert!(report.reflog_enabled);
    assert!(!report.reflog.is_empty());
    assert!(
        report.reflog.iter().all(|e| !e.reference.is_empty()),
        "reflog entries must carry their reference name"
    );

    // Create a commit on a side branch, then delete the branch so the commit
    // is only reachable via the reflog.
    repo.git(&["checkout", "-q", "-b", "doomed"]);
    repo.write("doomed.txt", "lost work\n");
    repo.commit("feat: doomed work");
    repo.git(&["checkout", "-q", "main"]);
    assert!(
        repo.git(&["branch", "-D", "doomed"]).is_some(),
        "delete branch"
    );

    let unreachable = find_unreachable_commits(&gix, None).expect("unreachable");
    assert!(
        !unreachable.is_empty(),
        "deleted branch should leave unreachable commits"
    );
}
