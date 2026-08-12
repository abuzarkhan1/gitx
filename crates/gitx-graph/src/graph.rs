use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeType {
    File,
    Directory,
    Module,
    Function,
    Struct,
    Trait,
}

#[derive(Debug, Clone)]
pub struct NodeData {
    pub name: String,
    pub path: PathBuf,
    pub node_type: NodeType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EdgeType {
    Contains,
    Calls,
    Imports,
    Implements,
    References,
}

#[derive(Debug, Clone)]
pub struct EdgeData {
    pub edge_type: EdgeType,
    pub weight: u32,
}

pub struct CodeGraph {
    pub graph: DiGraph<NodeData, EdgeData>,
    pub node_indices: HashMap<PathBuf, NodeIndex>,
}

impl Default for CodeGraph {
    fn default() -> Self {
        Self {
            graph: DiGraph::new(),
            node_indices: HashMap::new(),
        }
    }
}

impl CodeGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(
        &mut self,
        path: impl AsRef<Path>,
        name: String,
        node_type: NodeType,
    ) -> NodeIndex {
        let path = path.as_ref().to_path_buf();
        if let Some(&index) = self.node_indices.get(&path) {
            return index;
        }

        let data = NodeData {
            name,
            path: path.clone(),
            node_type,
        };
        let index = self.graph.add_node(data);
        self.node_indices.insert(path, index);
        index
    }

    pub fn add_edge(&mut self, from: NodeIndex, to: NodeIndex, edge_type: EdgeType, weight: u32) {
        self.graph
            .add_edge(from, to, EdgeData { edge_type, weight });
    }

    pub fn get_node(&self, path: impl AsRef<Path>) -> Option<NodeIndex> {
        self.node_indices.get(path.as_ref()).copied()
    }
}

/// Build the HEAD code graph: file + directory nodes, Contains edges,
/// heuristic Imports edges, and Call edges derived from the symbol
/// extractor (docs/02 V1 "stronger architecture graph", docs/21 Stage 6).
/// Deterministic order; read-only (Git trees only).
pub fn build_head_code_graph(repo: &gitx_git::Repository) -> anyhow::Result<CodeGraph> {
    let head = repo.head_commit_id()?;
    let head_commit = repo.find_commit(head)?;
    let tree_id = head_commit.tree_id;
    let blobs = repo.list_blobs(tree_id)?;
    let mut graph = CodeGraph::new();
    let mut dirs: std::collections::HashSet<String> = std::collections::HashSet::new();

    // Phase 1: file + directory nodes and Contains edges. All nodes exist
    // before any import/call edge is resolved, so edge detection is
    // independent of the blob iteration order.
    for path in &blobs {
        let path_str = path.display().to_string();
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path_str.clone());
        graph.add_node(path, name, NodeType::File);
        let mut parent = path.parent();
        while let Some(p) = parent {
            let dir = p.display().to_string();
            if dir != "." && !dir.is_empty() && dirs.insert(dir.clone()) {
                let dname = p
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| dir.clone());
                graph.add_node(p, dname, NodeType::Directory);
            }
            parent = p.parent();
        }
        if let Some(par) = path.parent() {
            let par_str = par.display().to_string();
            if par_str != "."
                && !par_str.is_empty()
                && let (Some(a), Some(b)) = (graph.get_node(par), graph.get_node(path))
            {
                graph.add_edge(a, b, EdgeType::Contains, 1);
            }
        }
    }

    // Symbol index: function/method name -> owning file (first match wins
    // while iterating sorted paths, so it is deterministic).
    let symbols = gitx_analysis::symbols::extract_symbols_from_tree(repo, tree_id)?;
    let mut owner: std::collections::HashMap<String, std::path::PathBuf> =
        std::collections::HashMap::new();
    for (path, syms) in &symbols {
        for s in syms {
            if matches!(s.kind.as_str(), "Function" | "Method") {
                owner.entry(s.name.clone()).or_insert_with(|| path.clone());
            }
        }
    }

    // Phase 2: heuristic import + call edges (read-only; Git trees only).
    for path in &blobs {
        let Ok(Some(bytes)) = repo.blob_at_path(tree_id, path) else {
            continue;
        };
        let content = String::from_utf8_lossy(&bytes);
        let ext = path
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        let imports: Vec<String> = match ext.as_str() {
            "rs" => content
                .lines()
                .filter(|l| l.trim_start().starts_with("use "))
                .map(|l| {
                    l.trim()
                        .trim_start_matches("use ")
                        .split("::")
                        .next()
                        .unwrap_or("")
                        .to_string()
                })
                .collect(),
            "js" | "ts" | "jsx" | "tsx" | "mjs" | "cjs" => content
                .lines()
                .filter_map(|l| {
                    let t = l.trim();
                    if let Some(rest) = t.strip_prefix("import ")
                        && rest.contains("from")
                    {
                        rest.split("from")
                            .nth(1)
                            .map(|s| s.trim().trim_matches(['\'', '\"', ';']).to_string())
                    } else {
                        t.strip_prefix("require(").map(|rest| {
                            rest.trim().trim_matches(['\'', '\"', ')', ';']).to_string()
                        })
                    }
                })
                .filter(|i| i.starts_with('.') || i.starts_with('/'))
                .collect(),
            "py" => content
                .lines()
                .filter_map(|l| {
                    let t = l.trim();
                    if let Some(rest) = t.strip_prefix("from ")
                        && let Some((module, _)) = rest.split_once(" import")
                    {
                        Some(module.trim().to_string())
                    } else if let Some(rest) = t.strip_prefix("import ")
                        && !rest.starts_with('(')
                    {
                        Some(rest.split_whitespace().next().unwrap_or("").to_string())
                    } else {
                        None
                    }
                })
                .collect(),
            "go" => content
                .lines()
                .filter_map(|l| {
                    let t = l.trim();
                    t.strip_prefix("import ")
                        .map(|s| s.trim().trim_matches(['\'', '\"']).to_string())
                        .or_else(|| {
                            t.strip_prefix('\t')
                                .filter(|_| t.starts_with('\"'))
                                .map(|s| s.trim().trim_matches('\"').to_string())
                        })
                })
                .filter(|i| i.contains('/'))
                .collect(),
            _ => Vec::new(),
        };
        for imp in imports {
            if imp.is_empty() {
                continue;
            }
            // Relative imports resolve against the importing file's directory;
            // absolute-ish module paths are kept as-is (unresolvable -> skip).
            let target = if imp.starts_with('.') {
                let base = path.parent().unwrap_or_else(|| std::path::Path::new("/"));
                let joined = base.join(&imp);
                let norm = normalize(&joined.to_string_lossy());
                if repo
                    .list_blobs(tree_id)
                    .unwrap_or_default()
                    .iter()
                    .any(|p| p.starts_with(&norm))
                {
                    norm
                } else {
                    continue;
                }
            } else {
                continue; // external/unresolvable imports are out of scope
            };
            if let (Some(a), Some(b)) = (graph.get_node(path), graph.get_node(&target)) {
                graph.add_edge(a, b, EdgeType::Imports, 1);
            }
        }

        // Call edges: any `name(` occurrence where `name` is a function owned
        // by a *different* file (docs/02 V1). Occurrence counts are capped at
        // 100 per target so one hot caller cannot dominate.
        let mut calls: std::collections::BTreeMap<String, u32> = std::collections::BTreeMap::new();
        for (name, owner_path) in &owner {
            if *owner_path == *path {
                continue; // no self edges
            }
            let count = content.matches(&format!("{name}(")).count() as u32;
            if count > 0 {
                *calls.entry(owner_path.display().to_string()).or_insert(0) += count.min(100);
            }
        }
        for (target, weight) in calls {
            if let (Some(a), Some(b)) = (graph.get_node(path), graph.get_node(&target)) {
                graph.add_edge(a, b, EdgeType::Calls, weight);
            }
        }
    }
    Ok(graph)
}

/// Per-directory module summary for the TUI Graph view: (directory, file
/// count, import edge count, call edge count). Directories sorted by path.
pub fn module_summary(
    repo: &gitx_git::Repository,
) -> anyhow::Result<Vec<(String, usize, usize, usize)>> {
    let graph = build_head_code_graph(repo)?;
    let mut files: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut imports: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut calls: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for edge in graph.graph.edge_references() {
        let from = graph.graph[edge.source()].path.display().to_string();
        match edge.weight().edge_type {
            // Contains edges run from a directory node to a file node, so the
            // source path IS the directory.
            EdgeType::Contains => {
                *files.entry(from).or_insert(0) += 1;
            }
            EdgeType::Imports | EdgeType::Calls => {
                let dir = std::path::Path::new(&from)
                    .parent()
                    .map(|p| p.display().to_string())
                    .unwrap_or_default();
                match edge.weight().edge_type {
                    EdgeType::Imports => {
                        *imports.entry(dir).or_insert(0) += 1;
                    }
                    _ => {
                        *calls.entry(dir).or_insert(0) += 1;
                    }
                }
            }
            _ => {}
        }
    }
    let mut dirs: Vec<String> = files.keys().cloned().collect();
    dirs.sort();
    Ok(dirs
        .into_iter()
        .map(|d| {
            (
                d.clone(),
                files[&d],
                imports.get(&d).copied().unwrap_or(0),
                calls.get(&d).copied().unwrap_or(0),
            )
        })
        .collect())
}

/// Normalize a relative path string (remove `./` and resolve `..` segments).
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
