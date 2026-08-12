use crate::views::{common, theme};
use gitx_analysis::RepoAnalysis;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
};

pub fn render(
    f: &mut Frame,
    area: Rect,
    analysis: Option<&RepoAnalysis>,
    loading: bool,
    scroll: usize,
    selected: usize,
) -> usize {
    let rows: Vec<Line<'static>> = match analysis {
        None => common::panel_placeholder(loading, "repository analysis"),
        Some(a) if a.files.is_empty() => vec![theme::plain("No files analyzed.")],
        Some(a) => {
            let mut files: Vec<&gitx_analysis::FileAnalysis> = a.files.iter().collect();
            files.sort_by(|x, y| {
                y.risk
                    .partial_cmp(&x.risk)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            let mut out = Vec::new();
            for file in &files {
                let color = theme::severity_color(file.risk);
                let class_color = theme::class_color(file.classification);
                out.push(Line::from(vec![
                    Span::styled("⚠", Style::default().fg(color)),
                    Span::styled(
                        format!(" {:.0}/100  ", file.risk),
                        Style::default().fg(color).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!("{:<7}  ", file.classification),
                        Style::default().fg(class_color),
                    ),
                    Span::raw(file.path.display().to_string()),
                ]));
                out.push(theme::dim(format!(
                    "         changes {} | churn 30d {} | fixes {} | contributors {} | ownership {:.0}%",
                    file.metrics.change_frequency,
                    file.metrics.lines_added + file.metrics.lines_deleted,
                    file.metrics.bug_fix_count,
                    file.metrics.unique_contributors,
                    file.ownership_concentration
                )));
            }
            out
        }
    };
    common::render_scrollable(
        f,
        area,
        " Risk — evidence-backed maintenance risk (0–100) ",
        &rows,
        scroll,
        selected,
    )
}
