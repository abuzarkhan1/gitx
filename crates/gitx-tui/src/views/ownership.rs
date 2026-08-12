use crate::views::{common, theme};
use gitx_analysis::RepoAnalysis;
use ratatui::{Frame, layout::Rect, text::Line};

pub fn render(
    f: &mut Frame,
    area: Rect,
    analysis: Option<&RepoAnalysis>,
    scroll: usize,
    selected: usize,
) -> usize {
    let rows: Vec<Line<'static>> = match analysis {
        None => common::empty_rows("repository"),
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
            let mut out = Vec::new();
            for file in files.iter().take(60) {
                let top = file.author_lines.iter().max_by_key(|(_, v)| *v);
                let top_str = top
                    .map(|(a, l)| format!("{a} ({l} lines)"))
                    .unwrap_or_else(|| "-".into());
                let color = theme::severity_color(file.ownership_concentration);
                // Concentration bar (docs/08 Ownership): the bar length IS the
                // ownership %, colored by how risky the concentration is.
                out.push(theme::hbar(
                    format!(
                        "{}  ({} contributors)",
                        file.path.display(),
                        file.metrics.unique_contributors
                    ),
                    file.ownership_concentration,
                    24,
                    color,
                ));
                out.push(theme::dim(format!("         top contributor: {top_str}")));
            }
            if out.is_empty() {
                vec![
                    theme::plain("No ownership data (analysis unavailable)."),
                    theme::dim("Run: gitx refresh to build the repository index."),
                ]
            } else {
                out
            }
        }
    };
    common::render_scrollable(
        f,
        area,
        " Ownership — per-file concentration (higher = more risk) ",
        &rows,
        scroll,
        selected,
    )
}
