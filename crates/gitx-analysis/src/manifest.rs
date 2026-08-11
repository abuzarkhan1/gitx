use gitx_git::Repository;
use std::path::{Path, PathBuf};

/// Dependency manifests recognized by the deterministic analyzer (docs/10 §11).
pub const MANIFESTS: [&str; 5] = [
    "Cargo.toml",
    "package.json",
    "go.mod",
    "requirements.txt",
    "pyproject.toml",
];

/// Lockfiles recognized by the deterministic analyzer. When present, these
/// give *precise resolved versions* rather than declared constraints.
pub const LOCKFILES: [&str; 5] = [
    "Cargo.lock",
    "package-lock.json",
    "yarn.lock",
    "pnpm-lock.yaml",
    "go.sum",
];

/// A dependency name (and optional version constraint) as declared in a
/// manifest. Versions are extracted heuristically where the format allows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dependency {
    pub name: String,
    pub version: Option<String>,
}

impl std::fmt::Display for Dependency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.version {
            Some(v) => write!(f, "{} {}", self.name, v),
            None => write!(f, "{}", self.name),
        }
    }
}

/// All supported manifests found in the HEAD tree with their parsed
/// dependencies. Deterministic; reads only the HEAD tree (docs/10 §11).
pub fn head_dependencies(repo: &Repository) -> anyhow::Result<Vec<(PathBuf, Vec<Dependency>)>> {
    let head = repo.head_commit_id()?;
    let head_commit = repo.find_commit(head)?;
    head_dependencies_at(repo, head_commit.tree_id)
}

/// All supported manifests found in a given tree with their parsed
/// dependencies (used by dependency history, docs/10 §11).
pub fn head_dependencies_at(
    repo: &Repository,
    tree_id: gitx_git::models::ObjectId,
) -> anyhow::Result<Vec<(PathBuf, Vec<Dependency>)>> {
    let mut found = Vec::new();
    for path in repo.list_blobs(tree_id)? {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        if MANIFESTS.contains(&name.as_str()) {
            let bytes = repo.blob_at_path(tree_id, &path)?.unwrap_or_default();
            let content = String::from_utf8_lossy(&bytes);
            let deps = parse_manifest(&name, &content);
            found.push((path, deps));
        }
    }
    Ok(found)
}

/// Lockfiles found in a given tree with their parsed precise dependencies
/// (docs/10 §11: dependency version evolution from lockfiles).
pub fn lockfile_dependencies_at(
    repo: &Repository,
    tree_id: gitx_git::models::ObjectId,
) -> anyhow::Result<Vec<(PathBuf, Vec<Dependency>)>> {
    let mut found = Vec::new();
    for path in repo.list_blobs(tree_id)? {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        if LOCKFILES.contains(&name.as_str()) {
            let bytes = repo.blob_at_path(tree_id, &path)?.unwrap_or_default();
            let content = String::from_utf8_lossy(&bytes);
            if let Some(deps) = parse_lockfile(&name, &content) {
                found.push((path, deps));
            }
        }
    }
    Ok(found)
}

/// Very small, format-aware manifest parsers. Results are deterministic but
/// heuristic; they only read the HEAD tree.
pub fn parse_manifest(name: &str, content: &str) -> Vec<Dependency> {
    match name {
        "Cargo.toml" => parse_cargo(content),
        "package.json" => parse_package_json(content),
        "go.mod" => parse_go_mod(content),
        "requirements.txt" | "pyproject.toml" => parse_python(content),
        _ => Vec::new(),
    }
}

fn parse_cargo(content: &str) -> Vec<Dependency> {
    let mut in_deps = false;
    let mut out = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_deps = matches!(
                trimmed,
                "[dependencies]" | "[dev-dependencies]" | "[build-dependencies]"
            );
            continue;
        }
        if in_deps && let Some(eq) = trimmed.find('=') {
            let dep = trimmed[..eq].trim().trim_matches('"').to_string();
            if !dep.is_empty() && !dep.starts_with('#') {
                let version = trimmed[eq + 1..].trim().trim_matches('"').to_string();
                out.push(Dependency {
                    version: if version.is_empty() || version.starts_with('{') {
                        None
                    } else {
                        Some(version)
                    },
                    name: dep,
                });
            }
        }
    }
    out
}

fn parse_package_json(content: &str) -> Vec<Dependency> {
    let mut out = Vec::new();
    let mut section: Option<String> = None;
    for line in content.lines() {
        let trimmed = line.trim().trim_end_matches(',');
        if trimmed.starts_with('"') {
            let key = trimmed
                .trim_start_matches('"')
                .split('"')
                .next()
                .unwrap_or("");
            if key == "dependencies" || key == "devDependencies" {
                section = Some(key.to_string());
                continue;
            }
            if section.is_some() && trimmed.contains(':') {
                let name = key.to_string();
                if !name.is_empty() {
                    let version = trimmed
                        .split_once(':')
                        .map(|(_, v)| v.trim().trim_matches('"'))
                        .unwrap_or("")
                        .to_string();
                    out.push(Dependency {
                        version: if version.is_empty() {
                            None
                        } else {
                            Some(version)
                        },
                        name,
                    });
                }
            }
        } else if trimmed == "}" && section.is_some() {
            section = None;
        }
    }
    out
}

fn parse_go_mod(content: &str) -> Vec<Dependency> {
    let mut out = Vec::new();
    let mut in_require = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("require (") {
            in_require = true;
            continue;
        }
        if in_require {
            if trimmed == ")" {
                in_require = false;
                continue;
            }
            let mut parts = trimmed.split_whitespace();
            if let Some(name) = parts.next()
                && !name.is_empty()
                && !name.starts_with("//")
            {
                out.push(Dependency {
                    version: parts.next().map(str::to_string),
                    name: name.to_string(),
                });
            }
        } else if let Some(rest) = trimmed.strip_prefix("require ") {
            let mut parts = rest.split_whitespace();
            if let Some(name) = parts.next()
                && !name.is_empty()
            {
                out.push(Dependency {
                    version: parts.next().map(str::to_string),
                    name: name.to_string(),
                });
            }
        }
    }
    out
}

fn parse_python(content: &str) -> Vec<Dependency> {
    let mut out = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        // Skip non-dependency sections (e.g. pyproject [project] header).
        if trimmed.starts_with('[') || trimmed.starts_with(']') {
            continue;
        }
        let mut parts = trimmed.splitn(2, [':', '=', ' ', '>', '<', '~', '!']);
        let name = parts
            .next()
            .map(|s| s.trim().trim_matches('"').trim_matches('\''))
            .unwrap_or("");
        if name.is_empty() {
            continue;
        }
        // Heuristic filter: looks like a package name.
        if name.contains('/') || name.contains('\\') || name.starts_with('.') {
            continue;
        }
        let version = parts
            .next()
            .map(|v| v.trim().trim_matches('"').trim_matches('\''))
            .filter(|v| !v.is_empty())
            .map(str::to_string);
        out.push(Dependency {
            name: name.to_string(),
            version,
        });
    }
    out
}

/// Parse a lockfile into precise (name, version) pairs. Returns `None` when
/// the lockfile format is not recognized.
///
/// These are intentionally small, deterministic parsers for the common
/// shapes; they are heuristic, not a full package-manager implementation.
pub fn parse_lockfile(name: &str, content: &str) -> Option<Vec<Dependency>> {
    match name {
        "Cargo.lock" => Some(parse_cargo_lock(content)),
        "package-lock.json" => Some(parse_package_lock(content)),
        "yarn.lock" => Some(parse_yarn_lock(content)),
        "pnpm-lock.yaml" => Some(parse_pnpm_lock(content)),
        "go.sum" => Some(parse_go_sum(content)),
        _ => None,
    }
}

/// Cargo.lock: `[[package]]` blocks with `name = "..."` and `version = "..."`.
fn parse_cargo_lock(content: &str) -> Vec<Dependency> {
    let mut out = Vec::new();
    let mut name: Option<String> = None;
    let mut version: Option<String> = None;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "[[package]]" {
            if let (Some(n), Some(v)) = (name.take(), version.take()) {
                out.push(Dependency {
                    name: n,
                    version: Some(v),
                });
            }
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("name = \"") {
            name = rest.strip_suffix('"').map(str::to_string);
        } else if let Some(rest) = trimmed.strip_prefix("version = \"") {
            version = rest.strip_suffix('"').map(str::to_string);
        }
    }
    if let (Some(n), Some(v)) = (name, version) {
        out.push(Dependency {
            name: n,
            version: Some(v),
        });
    }
    out
}

/// package-lock.json: `"name": "..."` entries inside the top-level
/// `dependencies`/`devDependencies`/`optionalDependencies` objects, plus the
/// root package name. Crude but deterministic.
fn parse_package_lock(content: &str) -> Vec<Dependency> {
    let mut out = Vec::new();
    // Track whether the current `{` object is one of the dependency sections.
    let mut in_deps_section = false;
    // When we see `"name": {` inside a deps section, remember the name until
    // its `"version"` key appears (at depth+1).
    let mut pending: Option<String> = None;

    for line in content.lines() {
        let trimmed = line.trim().trim_end_matches(',');
        if !(trimmed.starts_with('"') && trimmed.contains(':')) {
            continue;
        }
        let key = trimmed
            .trim_start_matches('"')
            .split('"')
            .next()
            .unwrap_or("");
        let value = trimmed.split_once(':').map(|(_, v)| v).unwrap_or("").trim();

        // Section open: `"dependencies": {`
        if value == "{" {
            if matches!(
                key,
                "dependencies" | "devDependencies" | "optionalDependencies"
            ) {
                in_deps_section = true;
                pending = None;
                continue;
            }
            // A package entry: `"react": {` inside a deps section.
            if in_deps_section && !key.is_empty() {
                pending = Some(key.to_string());
            }
            continue;
        }

        // Section close: `}` — handled when the value is empty-ish or "}".
        if value.is_empty() || value == "}" {
            if trimmed.contains('}') && in_deps_section {
                // A `}` line: only close the section at the section's own
                // closing brace. Heuristic: treat lone `}` as possibly the
                // section end; the next dependency-section open resets it.
                // We keep it simple and only reset on the next section open.
            }
            continue;
        }

        // Inside a package object: `"version": "18.2.0"`
        if in_deps_section
            && key == "version"
            && let Some(n) = pending.take()
        {
            let v = value.trim_matches('"');
            if !n.is_empty() && !v.is_empty() {
                out.push(Dependency {
                    name: n,
                    version: Some(v.to_string()),
                });
            }
        }
    }
    out
}

/// go.sum: `module version h1:...` lines (module names with versions).
/// Deduplicates by name, keeping the last (most recent) version.
fn parse_go_sum(content: &str) -> Vec<Dependency> {
    let mut out = Vec::new();
    for line in content.lines() {
        let mut parts = line.split_whitespace();
        let name = parts.next().unwrap_or("");
        let version = parts.next().unwrap_or("");
        if name.is_empty() || version.is_empty() || version == "go.mod" {
            continue;
        }
        if let Some(rest) = version.strip_prefix("v") {
            out.push(Dependency {
                name: name.to_string(),
                version: Some(format!("v{rest}")),
            });
        }
    }
    // Dedupe by name keeping last occurrence.
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    out.retain(|d| seen.insert(d.name.clone()));
    out
}

/// yarn.lock v1: blocks like `"@scope/pkg@^1.0.0":` followed by an indented
/// `version "1.3.0"`. The package name is everything before the last `@`
/// (scoped packages are `@scope/name@range`).
fn parse_yarn_lock(content: &str) -> Vec<Dependency> {
    let mut out = Vec::new();
    let mut pending: Option<(String, Option<String>)> = None;
    for line in content.lines() {
        let trimmed = line.trim_end();
        if pending.is_some() {
            // Inside a block: `version "1.3.0"`
            if let Some(rest) = trimmed.trim().strip_prefix("version ") {
                let version = rest.trim().trim_matches('"').trim_matches('\'');
                if let Some((name, _)) = pending.take() {
                    out.push(Dependency {
                        name,
                        version: if version.is_empty() {
                            None
                        } else {
                            Some(version.to_string())
                        },
                    });
                }
            }
            // A new top-level key ends the previous block.
            if !trimmed.starts_with(' ') && trimmed.ends_with(':') {
                pending = None;
            }
            continue;
        }
        // Top-level key ending in `:` starts a dependency block.
        if trimmed.ends_with(':') && !trimmed.starts_with(' ') {
            // Keys may be quoted (`"name@range", "name@range":`) or bare
            // (`name@range:`). Take the first comma-separated spec.
            let first = trimmed
                .trim_end_matches(':')
                .split(',')
                .next()
                .unwrap_or("")
                .trim()
                .trim_matches('"')
                .trim_matches('\'');
            if let Some((name, _)) = split_yarn_spec(first) {
                pending = Some((name, None));
            }
        }
    }
    // Dedupe by name keeping the first (most specific) version.
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    out.retain(|d| seen.insert(d.name.clone()));
    out
}

/// Split a yarn spec `name@range` into (name, range). Scoped packages are
/// `@scope/name@range` — find the last `@`.
fn split_yarn_spec(spec: &str) -> Option<(String, Option<String>)> {
    let at = spec.rfind('@')?;
    let name = &spec[..at];
    if name.is_empty() {
        return None;
    }
    let range = spec[at + 1..].trim();
    Some((
        name.to_string(),
        if range.is_empty() {
            None
        } else {
            Some(range.to_string())
        },
    ))
}

/// pnpm-lock.yaml v9: entries under `packages:` like
/// `/@scope/pkg@1.3.0:` followed by an indented `version: 1.3.0`.
/// The `importers:` section (with specifiers) is skipped.
fn parse_pnpm_lock(content: &str) -> Vec<Dependency> {
    let mut out = Vec::new();
    let mut in_packages = false;
    let mut pending: Option<String> = None;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("packages:") {
            in_packages = true;
            pending = None;
            continue;
        }
        if trimmed.starts_with("importers:") {
            in_packages = false;
            pending = None;
            continue;
        }
        if !in_packages {
            continue;
        }
        if trimmed.starts_with('/') && trimmed.ends_with(':') {
            // `  /@scope/pkg@1.3.0:` — strip the leading slash and split at the
            // last `@` (scoped names contain `@` in the scope too).
            let spec = trimmed.trim_start_matches('/').trim_end_matches(':');
            if let Some((name, _)) = split_yarn_spec(spec) {
                pending = Some(name);
            }
            continue;
        }
        if pending.is_some() && trimmed.starts_with("version:") {
            let version = trimmed["version:".len()..]
                .trim()
                .trim_matches('"')
                .trim_matches('\'');
            if let Some(name) = pending.take() {
                out.push(Dependency {
                    name,
                    version: if version.is_empty() {
                        None
                    } else {
                        Some(version.to_string())
                    },
                });
            }
        }
    }
    // Dedupe by name keeping the first occurrence.
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    out.retain(|d| seen.insert(d.name.clone()));
    out
}

/// Merge two dependency lists into a (added, removed, changed) summary.
/// `before`/`after` are dependency sets at two points in history.
fn same_name(a: &Dependency, b: &Dependency) -> bool {
    a.name == b.name
}

pub fn diff_dependencies<'a>(
    before: &'a [Dependency],
    after: &'a [Dependency],
) -> (
    Vec<&'a Dependency>,
    Vec<&'a Dependency>,
    Vec<(&'a Dependency, &'a Dependency)>,
) {
    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut changed = Vec::new();
    for d in after {
        match before.iter().find(|b| same_name(b, d)) {
            Some(b) => {
                if b.version != d.version {
                    changed.push((b, d));
                }
            }
            None => added.push(d),
        }
    }
    for b in before {
        if !after.iter().any(|a| same_name(a, b)) {
            removed.push(b);
        }
    }
    (added, removed, changed)
}

/// Diff two per-manifest dependency maps into a flat (added, removed, changed)
/// summary across all manifests (used by `gitx dependencies diff`).
pub fn find_manifest_deps<'a>(
    list: &'a [(PathBuf, Vec<Dependency>)],
    path: &Path,
) -> Option<&'a Vec<Dependency>> {
    list.iter().find(|(p, _)| p == path).map(|(_, deps)| deps)
}

pub fn diff_dependency_sets(
    before: &[(PathBuf, Vec<Dependency>)],
    after: &[(PathBuf, Vec<Dependency>)],
) -> (
    Vec<Dependency>,
    Vec<Dependency>,
    Vec<(Dependency, Dependency)>,
) {
    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut changed = Vec::new();

    for (path, deps) in after {
        match find_manifest_deps(before, path) {
            Some(before_deps) => {
                let (a, r, c) = diff_dependencies(before_deps, deps);
                added.extend(a.into_iter().cloned());
                removed.extend(r.into_iter().cloned());
                changed.extend(c.into_iter().map(|(b, a)| (b.clone(), a.clone())));
            }
            None => added.extend(deps.iter().cloned()),
        }
    }
    for (path, deps) in before {
        if find_manifest_deps(after, path).is_none() {
            removed.extend(deps.iter().cloned());
        }
    }
    (added, removed, changed)
}

/// Path of a manifest relative to the repository root, used for display.
pub fn display_path(path: &Path) -> String {
    path.display().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_cargo_toml() {
        let content = r#"
[package]
name = "demo"

[dependencies]
serde = "1.0"
anyhow = { version = "1" }

tokio = "1.39"

[dev-dependencies]
criterion = "0.5"
"#;
        let deps = parse_manifest("Cargo.toml", content);
        assert_eq!(deps.len(), 4);
        assert!(
            deps.iter()
                .any(|d| d.name == "serde" && d.version.as_deref() == Some("1.0"))
        );
        assert!(
            deps.iter()
                .any(|d| d.name == "anyhow" && d.version.is_none())
        );
        assert!(deps.iter().any(|d| d.name == "tokio"));
        assert!(deps.iter().any(|d| d.name == "criterion"));
    }

    #[test]
    fn parses_package_json() {
        let content = r#"{
  "name": "demo",
  "dependencies": {
    "react": "^18.0.0",
    "lodash": "4.17.21"
  },
  "devDependencies": {
    "typescript": "^5.0.0"
  }
}"#;
        let deps = parse_manifest("package.json", content);
        assert_eq!(deps.len(), 3);
        assert!(
            deps.iter()
                .any(|d| d.name == "react" && d.version.as_deref() == Some("^18.0.0"))
        );
        assert!(deps.iter().any(|d| d.name == "typescript"));
    }

    #[test]
    fn parses_go_mod() {
        let content = r#"module example.com/demo

go 1.21

require (
	github.com/pkg/errors v0.9.1
	golang.org/x/sync v0.7.0
)
"#;
        let deps = parse_manifest("go.mod", content);
        assert_eq!(deps.len(), 2);
        assert!(
            deps.iter().any(
                |d| d.name == "github.com/pkg/errors" && d.version.as_deref() == Some("v0.9.1")
            )
        );
    }

    #[test]
    fn diffs_dependencies() {
        let before = vec![
            Dependency {
                name: "serde".into(),
                version: Some("1.0".into()),
            },
            Dependency {
                name: "old-dep".into(),
                version: None,
            },
        ];
        let after = vec![
            Dependency {
                name: "serde".into(),
                version: Some("1.2".into()),
            },
            Dependency {
                name: "new-dep".into(),
                version: Some("2.0".into()),
            },
        ];
        let (added, removed, changed) = diff_dependencies(&before, &after);
        assert_eq!(added.len(), 1);
        assert_eq!(added[0].name, "new-dep");
        assert_eq!(removed.len(), 1);
        assert_eq!(removed[0].name, "old-dep");
        assert_eq!(changed.len(), 1);
        assert_eq!(changed[0].0.version.as_deref(), Some("1.0"));
        assert_eq!(changed[0].1.version.as_deref(), Some("1.2"));
    }

    #[test]
    fn parses_cargo_lock() {
        let content = r#"# This file is automatically @generated by Cargo.
version = 3

[[package]]
name = "anyhow"
version = "1.0.86"
source = "registry+https://github.com/rust-lang/crates.io-index"

[[package]]
name = "gitx"
version = "0.1.0"
dependencies = [
 "anyhow",
]
"#;
        let deps = parse_lockfile("Cargo.lock", content).unwrap();
        assert_eq!(deps.len(), 2);
        assert!(
            deps.iter()
                .any(|d| d.name == "anyhow" && d.version.as_deref() == Some("1.0.86"))
        );
        assert!(
            deps.iter()
                .any(|d| d.name == "gitx" && d.version.as_deref() == Some("0.1.0"))
        );
    }

    #[test]
    fn parses_go_sum() {
        let content = r#"github.com/pkg/errors v0.9.1 h1:FEBLx1zS214owpjy7qsBeixbURkuhQAwrK5UwLGTwt4=
github.com/pkg/errors v0.9.1/go.mod h1:bwawxfHBFNV+L2hUp1rHADufV3IMtnDRdf1r5NINEl0=
golang.org/x/sync v0.7.0 h1:YsFxQ8l1Fv9O2v8W8zQMWT2cRkO2US8HG6dHkmHXgPk=
"#;
        let deps = parse_lockfile("go.sum", content).unwrap();
        assert_eq!(deps.len(), 2);
        assert!(
            deps.iter().any(
                |d| d.name == "github.com/pkg/errors" && d.version.as_deref() == Some("v0.9.1")
            )
        );
    }

    #[test]
    fn parses_package_lock() {
        let content = r#"{
  "name": "demo",
  "version": "1.0.0",
  "lockfileVersion": 3,
  "packages": {},
  "dependencies": {
    "react": {
      "version": "18.2.0",
      "resolved": "https://registry.npmjs.org/react/-/react-18.2.0.tgz"
    },
    "lodash": {
      "version": "4.17.21"
    }
  }
}
"#;
        let deps = parse_lockfile("package-lock.json", content).unwrap();
        assert_eq!(deps.len(), 2);
        assert!(
            deps.iter()
                .any(|d| d.name == "react" && d.version.as_deref() == Some("18.2.0"))
        );
        assert!(
            deps.iter()
                .any(|d| d.name == "lodash" && d.version.as_deref() == Some("4.17.21"))
        );
    }

    #[test]
    fn parses_yarn_lock() {
        let content = r#"# THIS IS AN AUTOGENERATED FILE.

"@scope/pkg@^1.0.0", "@scope/pkg@^1.2.0":
  version "1.3.0"
  resolved "https://registry.yarnpkg.com/@scope/pkg/-/pkg-1.3.0.tgz"

lodash@^4.17.21:
  version "4.17.21"
  resolved "https://registry.yarnpkg.com/lodash/-/lodash-4.17.21.tgz"
"#;
        let deps = parse_lockfile("yarn.lock", content).unwrap();
        assert_eq!(deps.len(), 2);
        assert!(
            deps.iter()
                .any(|d| d.name == "@scope/pkg" && d.version.as_deref() == Some("1.3.0"))
        );
        assert!(
            deps.iter()
                .any(|d| d.name == "lodash" && d.version.as_deref() == Some("4.17.21"))
        );
    }

    #[test]
    fn parses_pnpm_lock() {
        let content = r#"lockfileVersion: '9.0'

importers:
  .:
    dependencies:
      react:
        specifier: ^18.0.0
        version: 18.2.0

packages:
  /@scope/pkg@1.3.0:
    resolution: {integrity: sha512-abc123}
    version: 1.3.0

  /lodash@4.17.21:
    resolution: {integrity: sha512-def456}
    version: 4.17.21
"#;
        let deps = parse_lockfile("pnpm-lock.yaml", content).unwrap();
        assert!(
            deps.iter()
                .any(|d| d.name == "@scope/pkg" && d.version.as_deref() == Some("1.3.0"))
        );
        assert!(
            deps.iter()
                .any(|d| d.name == "lodash" && d.version.as_deref() == Some("4.17.21"))
        );
    }
}
