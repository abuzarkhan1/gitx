use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyManifest {
    pub language: String,
    pub package_manager: String,
    pub dependencies: HashMap<String, DependencyInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyInfo {
    pub version_req: String,
    pub resolved_version: Option<String>,
    pub is_dev: bool,
}

impl DependencyManifest {
    pub fn new(language: &str, package_manager: &str) -> Self {
        Self {
            language: language.to_string(),
            package_manager: package_manager.to_string(),
            dependencies: HashMap::new(),
        }
    }

    pub fn add_dependency(&mut self, name: &str, info: DependencyInfo) {
        self.dependencies.insert(name.to_string(), info);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DependencyHistory {
    // Map of commit hash or timestamp to manifest
    pub history: Vec<(String, DependencyManifest)>,
}

impl DependencyHistory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record(&mut self, commit_or_time: String, manifest: DependencyManifest) {
        self.history.push((commit_or_time, manifest));
    }
}
