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
