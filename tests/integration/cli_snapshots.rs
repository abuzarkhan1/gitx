//! Snapshot tests (docs/14 §5): CLI human-readable output compared against
//! checked-in `.snap` files. Timestamps, commit hashes and machine-specific
//! paths are normalized before comparison so snapshots are stable across
//! machines and runs.
//!
//! Update snapshots deliberately with `GITX_BLESS=1 cargo test -p gitx-tests
//! --test cli_snapshots` after reviewing the diff.

use std::path::{Path, PathBuf};
use std::process::Command;

#[path = "../common/mod.rs"]
mod common;
use common::FixtureRepo;

/// Locate the built `gitx` binary. Prefers cargo's per-package env var, then
/// falls back to the workspace target dir (present after `cargo build` or
/// `cargo test --workspace`). Tests skip when the binary is absent.
fn gitx_bin() -> Option<PathBuf> {
    let candidates = [
        std::env::var_os("CARGO_BIN_EXE_gitx").map(PathBuf::from),
        Some(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../target/debug/gitx")),
    ];
    candidates.into_iter().flatten().find(|p| p.is_file())
}

/// Normalize volatile output so snapshots are machine-independent (docs/14
/// §5): commit oids (7–40 hex runs), ISO timestamps, and the fixture's temp
/// directory path.
fn normalize(fixture: &Path, output: &str) -> String {
    let mut masked = String::with_capacity(output.len());
    let chars: Vec<char> = output.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i].is_ascii_hexdigit() {
            let start = i;
            while i < chars.len() && chars[i].is_ascii_hexdigit() {
                i += 1;
            }
            let run: String = chars[start..i].iter().collect();
            if run.len() >= 7 {
                masked.push_str("<oid>");
            } else {
                masked.push_str(&run);
            }
        } else {
            masked.push(chars[i]);
            i += 1;
        }
    }

    // ISO timestamps: YYYY-MM-DD.
    let mut out = String::with_capacity(masked.len());
    let chars: Vec<char> = masked.chars().collect();
    i = 0;
    while i < chars.len() {
        if i + 9 < chars.len()
            && chars[i].is_ascii_digit()
            && chars[i + 1].is_ascii_digit()
            && chars[i + 2].is_ascii_digit()
            && chars[i + 3].is_ascii_digit()
            && chars[i + 4] == '-'
            && chars[i + 5].is_ascii_digit()
            && chars[i + 6].is_ascii_digit()
            && chars[i + 7] == '-'
            && chars[i + 8].is_ascii_digit()
            && chars[i + 9].is_ascii_digit()
        {
            out.push_str("<date>");
            i += 10;
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }

    // Machine-specific paths (the fixture lives in a per-run temp dir).
    let out = out.replace(&fixture.display().to_string(), "<fixture>");

    // Analysis durations ("in 12 ms") are volatile across runs/machines.
    let mut out = String::with_capacity(out.len());
    let chars: Vec<char> = out.chars().collect();
    i = 0;
    while i < chars.len() {
        if chars[i].is_ascii_digit() {
            let start = i;
            while i < chars.len() && chars[i].is_ascii_digit() {
                i += 1;
            }
            let digits: String = chars[start..i].iter().collect();
            let rest: String = chars[i..].iter().take(3).collect();
            if rest == " ms" {
                out.push_str("<ms>");
                i += 3;
            } else {
                out.push_str(&digits);
            }
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    out
}

/// Run `gitx <args>` against the fixture and compare normalized stdout to the
/// checked-in snapshot.
fn check_snapshot(repo: &FixtureRepo, args: &[&str], name: &str) {
    let Some(bin) = gitx_bin() else {
        eprintln!("skipping: gitx binary not built (run `cargo build` first)");
        return;
    };
    let output = Command::new(&bin)
        .args(args)
        .arg("--repo")
        .arg(repo.path())
        .output()
        .expect("run gitx");
    assert!(
        output.status.success(),
        "gitx {args:?} failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let normalized = normalize(repo.path(), &String::from_utf8_lossy(&output.stdout));

    let snap_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("snapshots")
        .join(format!("{name}.snap"));
    if std::env::var_os("GITX_BLESS").is_some() || !snap_path.exists() {
        std::fs::create_dir_all(snap_path.parent().expect("snapshot dir")).expect("mkdir");
        std::fs::write(&snap_path, normalized).expect("write snapshot");
        eprintln!("wrote snapshot {}", snap_path.display());
        return;
    }
    let expected = std::fs::read_to_string(&snap_path).expect("read snapshot");
    assert_eq!(
        normalized,
        expected,
        "snapshot mismatch for `gitx {}` — re-run with GITX_BLESS=1 to update",
        args.join(" ")
    );
}

#[test]
fn stats_snapshot() {
    let Some(repo) = FixtureRepo::new("snap-stats") else {
        eprintln!("skipping: git CLI unavailable");
        return;
    };
    repo.write("src/lib.rs", "pub fn hello() {}\n");
    repo.commit("feat: initial scaffold");
    repo.write("src/lib.rs", "pub fn hello() { println!(\"hi\"); }\n");
    repo.commit("fix: hello prints");
    repo.write("README.md", "# Sample\n");
    repo.commit("docs: add readme");
    check_snapshot(&repo, &["stats"], "stats");
}

#[test]
fn contributors_snapshot() {
    let Some(repo) = FixtureRepo::new("snap-contributors") else {
        eprintln!("skipping: git CLI unavailable");
        return;
    };
    repo.write("main.rs", "fn main() {}\n");
    repo.commit("feat: scaffold");
    repo.write("src/lib.rs", "pub fn helper() {}\n");
    repo.commit("feat: helper");
    check_snapshot(&repo, &["contributors"], "contributors");
}

#[test]
fn health_snapshot() {
    let Some(repo) = FixtureRepo::new("snap-health") else {
        eprintln!("skipping: git CLI unavailable");
        return;
    };
    repo.write("src/lib.rs", "pub fn hello() {}\n");
    repo.commit("feat: initial");
    repo.write("src/lib.rs", "pub fn hello() { println!(\"hi\"); }\n");
    repo.commit("fix: prints");
    check_snapshot(&repo, &["health"], "health");
}
