use std::path::Path;

/// Optional Tree-sitter abstraction for deeper syntactic analysis.
pub trait TreeSitterParser {
    fn parse_file(&self, path: &Path) -> Result<(), String>;
    // Future abstractions can be added here
}

pub struct DummyParser;

impl TreeSitterParser for DummyParser {
    fn parse_file(&self, _path: &Path) -> Result<(), String> {
        // Dummy implementation since it's an optional feature.
        Ok(())
    }
}
