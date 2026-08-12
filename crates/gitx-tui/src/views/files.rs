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
                let churn = file.metrics.lines_added + file.metrics.lines_deleted;
                format!(
                    "{:>5.1} {:<7}  {}  {:>4} changes  {:>6} churn  {:>2} authors  {:.0}% owned",
                    file.hotspot,
                    file.classification,
                    file.path.display(),
                    file.metrics.change_frequency,
                    churn,
                    file.metrics.unique_contributors,
                    file.ownership_concentration
                )
            })
            .collect(),
    };
    common::render_scrollable(
        f,
        area,
        " Files (hotspot | class) ",
        &rows,
        scroll,
        selected,
    )
}
