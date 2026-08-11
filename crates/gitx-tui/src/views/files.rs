use crate::views::common;
use gitx_analysis::RepoAnalysis;
use ratatui::{Frame, layout::Rect};

pub fn render(
    f: &mut Frame,
    area: Rect,
    analysis: Option<&RepoAnalysis>,
    scroll: usize,
    selected: usize,
) -> usize {
    let rows: Vec<String> = match analysis {
        None => vec!["No repository loaded.".to_string()],
        Some(a) if a.files.is_empty() => vec!["No files analyzed.".to_string()],
        Some(a) => a
            .files
            .iter()
            .map(|file| {
                format!(
                    "{}  {} commits  +{} −{}  {} contributors",
                    file.path.display(),
                    file.metrics.change_frequency,
                    file.metrics.lines_added,
                    file.metrics.lines_deleted,
                    file.metrics.unique_contributors
                )
            })
            .collect(),
    };
    common::render_scrollable(f, area, " Files ", &rows, scroll, selected)
}
