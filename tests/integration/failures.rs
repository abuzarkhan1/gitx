//! Failure-path tests (docs/14 §9): corrupted index, missing `.git`, invalid
//! paths, and malformed configuration must produce documented errors and
//! exit codes instead of panics.

use std::path::{Path, PathBuf};
use std::process::Command;

#[path = "../common/mod.rs"]
mod common;
use common::FixtureRepo;

/// Locate the built `gitx` binary; skip when absent (same policy as the
/// snapshot tests).
fn gitx_bin() -> Option<PathBuf> {
    let candidates = [
        std::env::var_os("CARGO_BIN_EXE_gitx").map(PathBuf::from),
        Some(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../target/debug/gitx")),
    ];
    candidates.into_iter().flatten().find(|p| p.is_file())
}

fn run(bin: &Path, args: &[&str], cwd: &Path) -> (i32, String, String) {
    let out = Command::new(bin)
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("run gitx");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
    )
}

#[test]
fn corrupted_index_is_reported_and_errors() {
    let Some(bin) = gitx_bin() else {
        eprintln!("skipping: gitx binary not built");
        return;
    };
    let Some(repo) = FixtureRepo::new("fail-corrupt") else {
        eprintln!("skipping: git CLI unavailable");
        return;
    };
    repo.write("main.rs", "fn main() {}\n");
    repo.commit("feat: scaffold");

    // Create an index and corrupt it.
    assert!(
        run(&bin, &["scan"], repo.path()).0 == 0,
        "scan should succeed"
    );
    let index = repo.path().join(".git/gitx/index.sqlite");
    std::fs::write(&index, b"this is not a sqlite database").expect("corrupt index");

    let (code, _, err) = run(&bin, &["index", "status"], repo.path());
    assert_eq!(code, 5, "corrupt index must exit 5, stderr: {err}");
    assert!(
        err.contains("index corrupt"),
        "must say the index is corrupt, got: {err}"
    );
}

#[test]
fn missing_git_directory_is_not_a_repo() {
    let Some(bin) = gitx_bin() else {
        eprintln!("skipping: gitx binary not built");
        return;
    };
    let dir = std::env::temp_dir().join(format!("gitx-it-nogit-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("mkdir");
    let (code, _, err) = run(&bin, &["info"], &dir);
    assert_eq!(code, 4, "missing .git must exit 4, stderr: {err}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn invalid_repo_path_is_an_error() {
    let Some(bin) = gitx_bin() else {
        eprintln!("skipping: gitx binary not built");
        return;
    };
    let (code, _, err) = run(
        &bin,
        &["--repo", "/nonexistent/gitx-dir", "info"],
        Path::new("/"),
    );
    assert_ne!(code, 0, "invalid --repo must fail, stderr: {err}");
    assert!(err.contains("cannot open repository"), "stderr: {err}");
}

#[test]
fn malformed_config_is_reported() {
    let Some(bin) = gitx_bin() else {
        eprintln!("skipping: gitx binary not built");
        return;
    };
    let Some(repo) = FixtureRepo::new("fail-config") else {
        eprintln!("skipping: git CLI unavailable");
        return;
    };
    repo.write("main.rs", "fn main() {}\n");
    repo.commit("feat: scaffold");

    let bad_config = repo.path().join("gitx.toml");
    std::fs::write(&bad_config, "this is not [valid toml =").expect("write bad config");

    let (code, _, err) = run(&bin, &["config", "show"], repo.path());
    assert_ne!(code, 0, "malformed config must fail, stderr: {err}");
    assert!(
        err.contains("config error") || err.contains("invalid config"),
        "stderr: {err}"
    );
}

#[test]
fn invalid_arguments_are_rejected() {
    let Some(bin) = gitx_bin() else {
        eprintln!("skipping: gitx binary not built");
        return;
    };
    let Some(repo) = FixtureRepo::new("fail-arg") else {
        eprintln!("skipping: git CLI unavailable");
        return;
    };
    repo.write("main.rs", "fn main() {}\n");
    repo.commit("feat: scaffold");

    // No such branch inside a real repository.
    let (code, _, err) = run(&bin, &["branch", "no-such-branch-xyz"], repo.path());
    assert_eq!(code, 2, "invalid argument must exit 2, stderr: {err}");
}
