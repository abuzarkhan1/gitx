use crate::views::common;
use gitx_analysis::manifest::Dependency;
use ratatui::{Frame, layout::Rect};
use std::path::PathBuf;

pub fn render(
    f: &mut Frame,
    area: Rect,
    dependencies: Option<&[(PathBuf, Vec<Dependency>)]>,
    scroll: usize,
    selected: usize,
) -> usize {
    let rows: Vec<String> = match dependencies {
        None => vec!["No repository loaded.".to_string()],
        Some([]) => vec!["No supported dependency manifests found in HEAD.".to_string()],
        Some(list) => {
            let mut out = Vec::new();
            for (path, deps) in list {
                out.push(path.display().to_string());
                for dep in deps {
                    match &dep.version {
                        Some(v) => out.push(format!("    {} {v}", dep.name)),
                        None => out.push(format!("    {}", dep.name)),
                    }
                }
            }
            out
        }
    };
    common::render_scrollable(f, area, " Dependencies ", &rows, scroll, selected)
}
