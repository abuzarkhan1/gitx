//! Code-content search (docs/11 §2, §4–§5): substring search over file
//! contents, bounded to the current working tree with a HEAD-tree fallback
//! for bare repositories or missing worktrees. Never streams every
//! historical blob (docs/11 §5); results are capped and binary files are
//! skipped so a big repository cannot balloon the search.
//!
//! Shared by the CLI (`gitx search --code`) and the TUI search service so
//! the bounds and the HEAD-tree fallback behave identically everywhere.

use crate::result::{EntityType, SearchResult};
use gitx_git::Repository;
use std::path::Path;

/// Where the code matches came from (docs/11 §5): the working tree, or the
/// HEAD-tree fallback when there is no working tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodeSource {
    Worktree,
    HeadTree,
}

impl std::fmt::Display for CodeSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CodeSource::Worktree => write!(f, "working tree"),
            CodeSource::HeadTree => write!(f, "HEAD tree"),
        }
    }
}

/// Outcome of a bounded code-content search.
#[derive(Debug)]
pub struct CodeSearchOutcome {
    pub results: Vec<SearchResult>,
    pub source: CodeSource,
    /// Files scanned (bounded): shows the scale and confirms the bound was
    /// honored (docs/11 §5).
    pub files_scanned: usize,
    /// True when the result cap was hit — callers surface a “capped” note.
    pub truncated: bool,
}

/// Default result cap (docs/11 §5 bounds; matches the CLI's historical cap).
pub const DEFAULT_CAP: usize = 50;

/// Search file contents for `query`, bounded. Prefers the working tree and
/// falls back to the HEAD tree when the workdir is unavailable (bare repos,
/// missing checkout) so code search still answers (docs/11 §2 “optionally
/// indexed snapshots”).
pub fn search_code_content(repo: &Repository, query: &str, cap: usize) -> CodeSearchOutcome {
    let cap = cap.max(1);
    let mut results = Vec::new();
    let mut files_scanned = 0usize;
    let mut truncated = false;

    if let Some(work_dir) = repo.work_dir() {
        let mut outcome = WorktreeSearch {
            results: &mut results,
            query,
            cap,
            files_scanned: &mut files_scanned,
            truncated: &mut truncated,
        };
        walk_worktree(work_dir, work_dir, &mut outcome, 0);
        if !results.is_empty() || work_dir.exists() {
            return CodeSearchOutcome {
                results,
                source: CodeSource::Worktree,
                files_scanned,
                truncated,
            };
        }
    }

    // HEAD-tree fallback (bare repositories / no checkout).
    let head_fallback = repo.head_commit_id().ok().and_then(|head| {
        let commit = repo.find_commit(head).ok()?;
        let entries = repo.tree_entries(commit.tree_id).ok()?;
        Some((commit.tree_id, entries))
    });
    if let Some((tree_id, entries)) = head_fallback {
        // Bounded scan: at most 2,000 blobs from the HEAD tree.
        for (path, _oid) in entries.into_iter().take(2000) {
            if results.len() >= cap {
                truncated = true;
                break;
            }
            files_scanned += 1;
            if is_binary_ext(&path) {
                continue;
            }
            let Ok(Some(bytes)) = repo.blob_at_path(tree_id, &path) else {
                continue;
            };
            if has_nul(&bytes) {
                continue;
            }
            let Ok(text) = String::from_utf8(bytes) else {
                continue;
            };
            if let Some(line) = text.lines().position(|l| l.contains(query)) {
                push_match(&mut results, &path, line, &text);
            }
        }
        if results.len() >= cap {
            truncated = true;
        }
        return CodeSearchOutcome {
            results,
            source: CodeSource::HeadTree,
            files_scanned,
            truncated,
        };
    }

    CodeSearchOutcome {
        results,
        source: CodeSource::Worktree,
        files_scanned,
        truncated,
    }
}

/// Bounded recursive walk over the working tree.
struct WorktreeSearch<'a> {
    results: &'a mut Vec<SearchResult>,
    query: &'a str,
    cap: usize,
    files_scanned: &'a mut usize,
    truncated: &'a mut bool,
}

fn walk_worktree(dir: &Path, root: &Path, s: &mut WorktreeSearch, depth: usize) {
    if depth > 12 || s.results.len() >= s.cap {
        if s.results.len() >= s.cap {
            *s.truncated = true;
        }
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        if s.results.len() >= s.cap {
            *s.truncated = true;
            break;
        }
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        // Never follow symlinks: a symlink could point outside the repository
        // root (docs/15 §5 symlink traversal defense).
        let file_type = entry
            .file_type()
            .ok()
            .or_else(|| std::fs::symlink_metadata(&path).map(|m| m.file_type()).ok());
        let Some(file_type) = file_type else { continue };
        if file_type.is_symlink() {
            continue;
        }
        if name == ".git" || (name.starts_with('.') && file_type.is_dir()) {
            continue;
        }
        if file_type.is_dir() {
            walk_worktree(&path, root, s, depth + 1);
            continue;
        }
        if is_binary_ext(&path) {
            continue;
        }
        *s.files_scanned += 1;
        let Ok(bytes) = std::fs::read(&path) else {
            continue;
        };
        if has_nul(&bytes) {
            continue;
        }
        let Ok(text) = String::from_utf8(bytes) else {
            continue;
        };
        let rel = path.strip_prefix(root).unwrap_or(&path);
        if let Some(line) = text.lines().position(|l| l.contains(s.query)) {
            push_match(s.results, rel, line, &text);
        }
    }
}

/// Append one code match: `path:line` as the display name, the matching line
/// as context.
fn push_match(results: &mut Vec<SearchResult>, path: &Path, line: usize, text: &str) {
    let ctx = text
        .lines()
        .nth(line)
        .map(|l| l.trim().chars().take(120).collect::<String>())
        .unwrap_or_default();
    results.push(SearchResult {
        entity_type: EntityType::Code,
        id: path.display().to_string(),
        display_name: format!("{}:{line}", path.display()),
        match_context: Some(ctx),
        score_if_applicable: None,
        recent_ts: None,
    });
}

/// Obvious binary extensions skipped during content scans (docs/11 §5).
fn is_binary_ext(path: &Path) -> bool {
    path.extension()
        .map(|e| {
            matches!(
                e.to_string_lossy().as_ref(),
                "png"
                    | "jpg"
                    | "jpeg"
                    | "gif"
                    | "ico"
                    | "pdf"
                    | "zip"
                    | "gz"
                    | "tar"
                    | "wasm"
                    | "woff"
                    | "ttf"
                    | "lock"
                    | "sqlite"
                    | "db"
            )
        })
        .unwrap_or(false)
}

/// Cheap binary sniff: a NUL byte in the first 8 KiB.
fn has_nul(bytes: &[u8]) -> bool {
    bytes.iter().take(8192).any(|&b| b == 0)
}

/// Convert a code match to a canonical display path string (forward slashes),
/// used by callers that key results by path.
pub fn canonical_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_path_normalizes_separators() {
        assert_eq!(canonical_path(Path::new("src/foo.rs")), "src/foo.rs");
        #[cfg(windows)]
        assert_eq!(canonical_path(Path::new("src\\foo.rs")), "src/foo.rs");
    }

    #[test]
    fn binary_extension_skipped() {
        assert!(is_binary_ext(Path::new("img/logo.png")));
        assert!(!is_binary_ext(Path::new("src/main.rs")));
    }

    #[test]
    fn nul_sniff() {
        assert!(has_nul(&[b'a', 0, b'b']));
        assert!(!has_nul(&b"hello world"[..]));
    }

    #[test]
    fn push_match_builds_context() {
        let mut results = Vec::new();
        // Line numbers are 0-indexed (from `str::lines().position`).
        push_match(
            &mut results,
            Path::new("a.txt"),
            1,
            "one\ntwo matches here\nthree",
        );
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].display_name, "a.txt:1");
        assert_eq!(
            results[0].match_context.as_deref(),
            Some("two matches here")
        );
    }
}
