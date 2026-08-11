use petgraph::graph::{DiGraph, NodeIndex};
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
