//! Test fixtures: build real Git repositories with the `git` CLI so the
//! gix-based crates can be exercised end-to-end (docs/14 testing strategy).
//!
//! `git` is used only to *create* fixtures; the code under test never shells
//! out to git.

use std::path::{Path, PathBuf};
use std::process::Command;

pub struct FixtureRepo {
    pub dir: PathBuf,
}

impl FixtureRepo {
    /// Create a fresh, uniquely-named repository under the system temp dir.
    /// Returns `None` if `git` is unavailable (tests then skip).
    pub fn new(name: &str) -> Option<Self> {
        let dir = std::env::temp_dir().join(format!(
            "gitx-it-{name}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).ok()?;
        let repo = Self { dir };
        repo.git(&["init", "-q", "-b", "main"])?;
        repo.git(&["config", "user.email", "it@example.com"])?;
        repo.git(&["config", "user.name", "IT Tester"])?;
        Some(repo)
    }

    pub fn path(&self) -> &Path {
        &self.dir
    }

    pub fn write(&self, rel: &str, content: &str) {
        let path = self.dir.join(rel);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("create fixture dir");
        }
        std::fs::write(path, content).expect("write fixture file");
    }

    /// `git add -A` + commit.
    pub fn commit(&self, message: &str) {
        assert!(self.git(&["add", "-A"]).is_some(), "git add failed");
        assert!(
            self.git(&["commit", "-qm", message]).is_some(),
            "git commit failed for: {message}"
        );
    }

    pub fn git(&self, args: &[&str]) -> Option<()> {
        let out = Command::new("git")
            .current_dir(&self.dir)
            .args(args)
            .output()
            .ok()?;
        if out.status.success() {
            Some(())
        } else {
            None
        }
    }
}

impl Drop for FixtureRepo {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// Standard small repository used by most integration tests.
/// Not every test target uses it; allow dead code when included by one that
/// doesn't.
#[allow(dead_code)]
pub fn sample_repo() -> Option<FixtureRepo> {
    let repo = FixtureRepo::new("sample")?;
    repo.write("src/lib.rs", "pub fn hello() {}\n");
    repo.write("main.rs", "fn main() {}\n");
    repo.commit("feat: initial scaffold");

    repo.write(
        "src/lib.rs",
        "pub fn hello() { println!(\"hi\"); }\npub fn world() {}\n",
    );
    repo.commit("fix: hello prints");

    repo.write("README.md", "# Sample\n");
    repo.commit("docs: add readme");
    Some(repo)
}
