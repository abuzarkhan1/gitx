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
        Some(a) => {
            let mut files: Vec<&gitx_analysis::FileAnalysis> = a
                .files
                .iter()
                .filter(|f| !f.author_lines.is_empty())
                .collect();
            files.sort_by(|x, y| {
                y.ownership_concentration
                    .partial_cmp(&x.ownership_concentration)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            files
                .iter()
                .map(|file| {
                    let top = file.author_lines.iter().max_by_key(|(_, v)| *v);
                    let top_str = top
                        .map(|(a, l)| format!("{a} ({l} lines)"))
                        .unwrap_or_else(|| "-".into());
                    format!(
                        "{:>5.1}%  {}  contributors {}  top: {}",
                        file.ownership_concentration,
                        file.path.display(),
                        file.metrics.unique_contributors,
                        top_str
                    )
                })
                .collect()
        }
    };
    common::render_scrollable(
        f,
        area,
        " Ownership concentration ",
        &rows,
        scroll,
        selected,
    )
}
