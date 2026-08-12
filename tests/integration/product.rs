//! Product-surface behavior (docs/01 UC-01, docs/16 §7, docs/10 §8):
//! end-to-end tests of the real `gitx` binary against fixture repositories —
//! no-arg behavior, health output, first-run auto-refresh, and config
//! gating. Built by `cargo build -p gitx-cli` (this crate shells out to the
//! binary; it does not depend on it).

#[path = "../common/mod.rs"]
mod common;

use common::FixtureRepo;
use std::process::Command;

fn bin() -> std::path::PathBuf {
    let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("..");
    p.push("target");
    p.push("debug");
    p.push(if cfg!(windows) { "gitx.exe" } else { "gitx" });
    p
}

#[test]
fn auto_refresh_builds_the_index_on_first_analytical_command() {
    let Some(repo) = FixtureRepo::new("product-autorefresh") else {
        return;
    };
    repo.write("src/lib.rs", "pub fn a() {}\npub fn b() {}\n");
    repo.commit("feat: initial");
    repo.write(
        "src/lib.rs",
        "pub fn a() { println!(\"x\"); }\npub fn b() {}\n",
    );
    repo.commit("fix: a prints");

    let out = Command::new(bin())
        .arg("--repo")
        .arg(repo.path())
        .arg("health")
        .output()
        .expect("gitx health runs");
    assert!(out.status.success(), "health failed: {:?}", out.status);

    let index = repo.path().join(".git/gitx/index.sqlite");
    assert!(
        index.exists(),
        "auto_refresh (default true) must create the index on first analysis"
    );
    let out = Command::new(bin())
        .arg("--repo")
        .arg(repo.path())
        .arg("index")
        .arg("status")
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(
        text.contains("2 commits"),
        "index should hold 2 commits: {text}"
    );
}

#[test]
fn auto_refresh_false_skips_index_build() {
    let Some(repo) = FixtureRepo::new("product-noautorefresh") else {
        return;
    };
    repo.write("README.md", "# demo\n");
    repo.commit("docs: readme");

    std::fs::write(
        repo.path().join("gitx.toml"),
        "[index]\nauto_refresh = false\n",
    )
    .expect("write repo config");

    let out = Command::new(bin())
        .arg("--repo")
        .arg(repo.path())
        .arg("health")
        .output()
        .expect("gitx health runs");
    assert!(out.status.success(), "health failed: {:?}", out.status);
    assert!(
        !repo.path().join(".git/gitx/index.sqlite").exists(),
        "auto_refresh=false must not create an index"
    );
}

#[test]
fn index_disabled_skips_scan_and_forces_live_analysis() {
    let Some(repo) = FixtureRepo::new("product-indexdisabled") else {
        return;
    };
    repo.write("README.md", "# demo\n");
    repo.commit("docs: readme");
    std::fs::write(repo.path().join("gitx.toml"), "[index]\nenabled = false\n")
        .expect("write repo config");

    let out = Command::new(bin())
        .arg("--repo")
        .arg(repo.path())
        .arg("scan")
        .output()
        .expect("gitx scan runs");
    assert!(out.status.success(), "scan must succeed with a message");
    assert!(
        !repo.path().join(".git/gitx/index.sqlite").exists(),
        "index.enabled=false must not create an index"
    );

    let out = Command::new(bin())
        .arg("--repo")
        .arg(repo.path())
        .arg("stats")
        .output()
        .expect("gitx stats runs");
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(
        text.contains("source: live"),
        "stats must be labeled live when the index is disabled: {text}"
    );
}

#[test]
fn default_limit_applies_to_hotspots() {
    let Some(repo) = FixtureRepo::new("product-defaultlimit") else {
        return;
    };
    for i in 0..10 {
        repo.write(&format!("src/f{i}.rs"), "pub fn x() {}\n");
        repo.commit(&format!("feat: file {i}"));
    }
    std::fs::write(
        repo.path().join("gitx.toml"),
        "[general]\ndefault_limit = 3\n",
    )
    .expect("write repo config");

    let out = Command::new(bin())
        .arg("--repo")
        .arg(repo.path())
        .arg("hotspots")
        .output()
        .expect("gitx hotspots runs");
    let text = String::from_utf8_lossy(&out.stdout);
    let rows = text
        .lines()
        .filter(|l| l.trim_start().starts_with(|c: char| c.is_ascii_digit()))
        .count();
    assert_eq!(
        rows, 3,
        "hotspots should honor default_limit=3 rows, got {rows}:\n{text}"
    );
}

#[test]
fn gitx_with_no_args_on_a_pipe_prints_a_snapshot() {
    let Some(repo) = FixtureRepo::new("product-noarg") else {
        return;
    };
    repo.write("src/lib.rs", "pub fn hello() {}\n");
    repo.commit("feat: initial");
    repo.write("src/lib.rs", "pub fn hello() { println!(\"hi\"); }\n");
    repo.commit("feat: print hi");

    let out = Command::new(bin())
        .arg("--repo")
        .arg(repo.path())
        .output()
        .expect("gitx runs");
    assert!(
        out.status.success(),
        "no-arg must exit 0, got {:?}",
        out.status
    );
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(
        text.contains("GitX") && text.contains("commits"),
        "piped no-arg must print a snapshot, got: {text}"
    );
    assert!(
        text.contains("2"),
        "fixture has 2 commits, snapshot should mention them: {text}"
    );
}

#[test]
fn gitx_tui_subcommand_exists_in_help() {
    let out = Command::new(bin())
        .arg("--help")
        .output()
        .expect("gitx --help runs");
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(
        text.contains("tui") && text.contains("interactive terminal"),
        "help must document `gitx tui`, got: {text}"
    );
}

#[test]
fn health_output_labels_bands_health_style() {
    let Some(repo) = FixtureRepo::new("product-healthbands") else {
        return;
    };
    repo.write("src/lib.rs", "pub fn hello() {}\n");
    repo.commit("feat: initial");

    let out = Command::new(bin())
        .arg("--repo")
        .arg(repo.path())
        .arg("health")
        .output()
        .expect("gitx health runs");
    assert!(out.status.success(), "gitx health failed: {:?}", out.status);
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(
        text.contains("POOR") && text.contains("EXCELLENT"),
        "health bands must use health semantics, got: {text}"
    );
    assert!(
        !text.contains("CRITICAL"),
        "health output must not reuse the risk CRITICAL band: {text}"
    );
    assert!(
        text.to_lowercase().contains("healthier"),
        "health output should state higher = healthier: {text}"
    );
}
