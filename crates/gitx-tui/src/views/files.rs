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
    scroll: usize,
    selected: usize,
) -> usize {
    let rows: Vec<Line<'static>> = match analysis {
        None => common::empty_rows("repository"),
        Some(a) if a.files.is_empty() => vec![theme::plain("No files analyzed.")],
        Some(a) => a
            .files
            .iter()
            .map(|file| {
                let churn = file.metrics.lines_added + file.metrics.lines_deleted;
                let score_color = theme::severity_color(file.hotspot);
                let class_color = theme::class_color(file.classification);
                Line::from(vec![
                    Span::styled(
                        format!("{:>5.1} ", file.hotspot),
                        Style::default()
                            .fg(score_color)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!("{:<7}  ", file.classification),
                        Style::default().fg(class_color),
                    ),
                    Span::raw(format!("{}", file.path.display())),
                    Span::styled(
                        format!(
                            "  {:>4} changes  {:>6} churn  {:>2} authors  {:.0}% owned",
                            file.metrics.change_frequency,
                            churn,
                            file.metrics.unique_contributors,
                            file.ownership_concentration
                        ),
                        Style::default().fg(ratatui::style::Color::DarkGray),
                    ),
                ])
            })
            .collect(),
    };
    common::render_scrollable(
        f,
        area,
        " Files — ranked by maintenance risk (0–100) ",
        &rows,
        scroll,
        selected,
    )
}
