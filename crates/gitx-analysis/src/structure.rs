//! Structural analysis (docs/10 §10, docs/21 Stage 6): heuristic import-edge
//! extraction at module (directory) granularity, dependency-direction change
//! detection between snapshots, and architecture milestone detection from the
//! commit history. Deterministic and read-only; the import scanner is the same
//! line-based heuristic used by `gitx graph`.

use gitx_git::Repository;
use std::collections::HashMap;
use std::path::PathBuf;

/// A directed module-level import edge: `from_dir` imports `to_dir` `weight`
/// times in a given tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleEdge {
    pub from_dir: String,
    pub to_dir: String,
    pub weight: u32,
}

/// Resolve heuristic import lines from a source file's content into
/// repo-relative file targets (only relative imports are resolvable; external
/// and standard-library imports are skipped).
fn imports_of(content: &str, ext: &str, path: &std::path::Path) -> Vec<String> {
    let mut out = Vec::new();
    match ext {
        "rs" => {
            for line in content.lines() {
                let t = line.trim();
                if let Some(rest) = t.strip_prefix("use ") {
                    let module = rest.split("::").next().unwrap_or("");
                    let module = module
                        .split_whitespace()
                        .next()
                        .unwrap_or("")
                        .trim_end_matches(';');
                    if !module.is_empty() {
                        out.push(format!("{}.rs", module.replace("::", "/")));
                    }
                }
            }
        }
        "js" | "ts" | "jsx" | "tsx" | "mjs" | "cjs" => {
            for line in content.lines() {
                let t = line.trim();
                if let Some(rest) = t.strip_prefix("import ")
                    && rest.contains("from")
                {
                    let target = rest
                        .split("from")
                        .nth(1)
                        .map(|s| {
                            s.trim()
                                .trim_matches(|c| c == '\'' || c == '"' || c == ';')
                                .to_string()
                        })
                        .unwrap_or_default();
                    if target.starts_with('.') {
                        out.push(target);
                    }
                } else if let Some(rest) = t.strip_prefix("require(") {
                    let target = rest
                        .trim()
                        .trim_matches(|c| c == '\'' || c == '"' || c == ')' || c == ';')
                        .to_string();
                    if target.starts_with('.') {
                        out.push(target);
                    }
                }
            }
        }
        "py" => {
            for line in content.lines() {
                let t = line.trim();
                if let Some(rest) = t.strip_prefix("from ")
                    && let Some((module, _)) = rest.split_once(" import")
                {
                    let module = module.trim().replace('.', "/");
                    out.push(format!("{module}.py"));
                } else if let Some(rest) = t.strip_prefix("import ")
                    && !rest.starts_with('(')
                {
                    let module = rest.split_whitespace().next().unwrap_or("");
                    let module = module.replace('.', "/");
                    out.push(format!("{module}.py"));
                }
            }
        }
        "go" => {
            // Go imports are full module paths; only repo-root-relative module
            // paths resolve, which requires the module name — skipped here
            // (external by default).
            let _ = path;
        }
        _ => {}
    }
    out
}

/// Normalize a relative path string (drop `.`/empty segments, resolve `..`).
fn normalize(p: &str) -> String {
    let mut parts: Vec<String> = Vec::new();
    for seg in p.split('/') {
        match seg {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            s => parts.push(s.to_string()),
        }
    }
    parts.join("/")
}

/// Module-level import edges for a tree (docs/10 §10). Heuristic: relative
/// imports only, resolved against the importing file's directory, aggregated
/// per (directory → directory) pair.
pub fn module_import_edges(
    repo: &Repository,
    tree_id: gitx_git::models::ObjectId,
) -> anyhow::Result<Vec<ModuleEdge>> {
    let blobs = repo.list_blobs(tree_id)?;
    let mut agg: HashMap<(String, String), u32> = HashMap::new();

    for path in &blobs {
        let ext = path
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        if !matches!(
            ext.as_str(),
            "rs" | "py" | "js" | "ts" | "jsx" | "tsx" | "mjs" | "cjs"
        ) {
            continue;
        }
        let Ok(Some(bytes)) = repo.blob_at_path(tree_id, path) else {
            continue;
        };
        let content = String::from_utf8_lossy(&bytes);
        let from_dir = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| ".".to_string());

        for imp in imports_of(&content, &ext, path) {
            // Resolve the target to a repo-relative file, then its directory.
            let target = if imp.starts_with('.') {
                let base = path.parent().unwrap_or_else(|| std::path::Path::new("/"));
                let joined = base.join(&imp);
                normalize(&joined.to_string_lossy())
            } else {
                continue; // external/unresolvable
            };
            let target = match_target(&blobs, &target);
            let Some(target) = target else { continue };
            let to_dir = target
                .parent()
                .filter(|p| !p.as_os_str().is_empty())
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| ".".to_string());
            if from_dir == to_dir {
                continue;
            }
            *agg.entry((from_dir.clone(), to_dir)).or_insert(0) += 1;
        }
    }

    let mut edges: Vec<ModuleEdge> = agg
        .into_iter()
        .map(|((from_dir, to_dir), weight)| ModuleEdge {
            from_dir,
            to_dir,
            weight,
        })
        .collect();
    edges.sort_by(|a, b| a.from_dir.cmp(&b.from_dir).then(a.to_dir.cmp(&b.to_dir)));
    Ok(edges)
}

/// Find the actual blob path that best matches `target` (a resolved import),
/// preferring exact, then a directory-relative prefix. `blobs` is the tree's
/// file list.
fn match_target(blobs: &[PathBuf], target: &str) -> Option<PathBuf> {
    // Exact match (maybe the target already includes an extension).
    if let Some(p) = blobs.iter().find(|p| p.to_string_lossy() == target) {
        return Some(p.clone());
    }
    // Try common extension suffixes for extension-less imports.
    for ext in ["rs", "py", "js", "ts", "jsx", "tsx", "mjs", "cjs"] {
        let candidate = format!("{target}.{ext}");
        if let Some(p) = blobs.iter().find(|p| p.to_string_lossy() == candidate) {
            return Some(p.clone());
        }
    }
    // Directory import: target/index.ext.
    for ext in ["js", "ts", "py", "rs"] {
        let candidate = format!("{target}/index.{ext}");
        if let Some(p) = blobs.iter().find(|p| p.to_string_lossy() == candidate) {
            return Some(p.clone());
        }
    }
    // Prefix match (closest directory).
    blobs
        .iter()
        .filter(|p| p.to_string_lossy().starts_with(&format!("{target}/")))
        .min_by_key(|p| p.components().count())
        .cloned()
}

/// Dependency-direction changes between two snapshots (docs/10 §10): for each
/// module pair with imports, report pairs where the direction flipped or a new
/// cross-module import appeared. Deterministic and labeled as structural
/// signals, not guarantees.
pub fn direction_changes(before: &[ModuleEdge], after: &[ModuleEdge]) -> Vec<String> {
    let dir_of = |edges: &[ModuleEdge]| -> HashMap<(String, String), u32> {
        let mut m = HashMap::new();
        for e in edges {
            *m.entry((e.from_dir.clone(), e.to_dir.clone())).or_insert(0) += e.weight;
        }
        m
    };
    let before = dir_of(before);
    let after = dir_of(after);

    let mut changes: Vec<String> = Vec::new();
    // Flip: B imports A before, A imports B after.
    for (a, b) in before.keys() {
        if after.get(&(b.clone(), a.clone())).is_some_and(|wa| wa > &0) {
            changes.push(format!("direction flipped: {b} now imports {a}"));
        }
    }
    // New cross-module import appeared.
    for ((a, b), w) in &after {
        if !before.contains_key(&(a.clone(), b.clone())) {
            changes.push(format!("new import: {a} → {b} ({w} edge(s))"));
        }
    }
    changes.sort();
    changes.dedup();
    changes.truncate(30);
    changes
}

/// One detected architectural milestone (docs/10 §10).
#[derive(Debug, Clone, serde::Serialize)]
pub struct Milestone {
    pub commit: String,
    pub time: i64,
    pub kind: &'static str,
    pub description: String,
}

/// Detect architectural milestones from the mainline (newest → oldest),
/// bounded to `max_commits`:
///
/// - **initial commit** — the oldest commit reached
/// - **first release tag** — the earliest commit carrying a tag
/// - **module added** — a new top-level directory with ≥2 files appears
/// - **structural refactor** — a commit whose change-set is dominated by
///   renames/moves
/// - **dependency-direction change** — net module import direction flips
///   between the newest snapshot and the snapshot at the walk's midpoint
pub fn architecture_milestones(
    repo: &Repository,
    max_commits: usize,
) -> anyhow::Result<Vec<Milestone>> {
    let head = repo.head_commit_id()?;
    let ids: Vec<gitx_git::models::ObjectId> = repo
        .rev_walk(head)?
        .collect::<gitx_git::Result<Vec<_>>>()?
        .into_iter()
        .take(max_commits)
        .collect();
    let mut milestones: Vec<Milestone> = Vec::new();

    // Tags by commit.
    let tags = repo.tags()?;

    // Walk oldest → newest for milestone ordering, keeping the newest tree's
    // module edges for the direction check.
    let mut seen_dirs: HashMap<String, usize> = HashMap::new();
    let mut direction_checked = false;
    let newest = ids.first().copied();
    let midpoint = ids.get(ids.len() / 2).copied();

    for (idx, id) in ids.iter().rev().enumerate() {
        let commit = repo.find_commit(*id)?;
        let short: String = id.to_string().chars().take(7).collect();
        let time = commit.author.time;

        // Initial commit: the oldest reached.
        if idx == 0 && !ids.is_empty() {
            milestones.push(Milestone {
                commit: short.clone(),
                time,
                kind: "initial_commit",
                description: "initial commit (oldest reached in the walk)".to_string(),
            });
        }

        // First release tag (earliest commit carrying a tag, walking oldest
        // first).
        if let Some(tag) = tags.iter().find(|t| t.target == *id) {
            milestones.push(Milestone {
                commit: short.clone(),
                time,
                kind: "release",
                description: format!("first release tag `{}`", tag.name),
            });
        }

        // Module added: a new top-level directory (≥2 files) appears.
        let blobs = repo.list_blobs(commit.tree_id)?;
        let mut dirs: HashMap<String, usize> = HashMap::new();
        for path in &blobs {
            let top = path
                .components()
                .next()
                .map(|c| c.as_os_str().to_string_lossy().to_string())
                .unwrap_or_else(|| ".".to_string());
            if top != "." {
                *dirs.entry(top).or_insert(0) += 1;
            }
        }
        for (dir, count) in &dirs {
            if *count >= 2 && !seen_dirs.contains_key(dir) {
                milestones.push(Milestone {
                    commit: short.clone(),
                    time,
                    kind: "module_added",
                    description: format!("module `{dir}/` appeared with {count} files"),
                });
            }
        }
        seen_dirs = dirs;

        // Structural refactor: renames/copies dominate the commit's changes.
        let parent_tree = commit
            .parents
            .first()
            .and_then(|p| repo.find_commit(*p).ok())
            .map(|p| p.tree_id);
        if let Some(pt) = parent_tree
            && let Ok(changes) = repo.diff_tree_to_tree(Some(pt), commit.tree_id)
            && changes.len() >= 5
        {
            let moves = changes
                .iter()
                .filter(|c| {
                    matches!(
                        c.change_type,
                        gitx_git::models::ChangeType::Renamed
                            | gitx_git::models::ChangeType::Copied
                    )
                })
                .count();
            if moves as f64 / changes.len() as f64 >= 0.4 {
                milestones.push(Milestone {
                    commit: short.clone(),
                    time,
                    kind: "structural_refactor",
                    description: format!("{moves}/{} changes are renames/moves", changes.len()),
                });
            }
        }

        // Dependency-direction changes between the newest snapshot and the
        // midpoint snapshot (once, using the two sampled trees).
        if !direction_checked && idx == ids.len().saturating_sub(1) / 2 {
            if let (Some(n), Some(m)) = (newest, midpoint)
                && n != m
            {
                let before =
                    module_import_edges(repo, repo.find_commit(m)?.tree_id).unwrap_or_default();
                let after =
                    module_import_edges(repo, repo.find_commit(n)?.tree_id).unwrap_or_default();
                for change in direction_changes(&before, &after) {
                    milestones.push(Milestone {
                        commit: short.clone(),
                        time,
                        kind: "dependency_direction",
                        description: change,
                    });
                }
            }
            direction_checked = true;
        }
    }

    milestones.sort_by_key(|m| (m.time, m.commit.clone()));
    Ok(milestones)
}
