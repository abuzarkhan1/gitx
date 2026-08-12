use crate::views::{common, theme};
use ratatui::{Frame, layout::Rect, style::Color, text::Line};

pub fn render(
    f: &mut Frame,
    area: Rect,
    text: Option<&[String]>,
    scroll: usize,
    selected: usize,
) -> usize {
    let rows: Vec<Line<'static>> = match text {
        None => common::empty_rows("detail"),
        Some([]) => vec![theme::plain("Nothing selected.")],
        Some(lines) => lines
            .iter()
            .map(|line| {
                // Diff-stats coloring (docs/08 #15): +green / -red so a
                // commit's insertions/deletions read at a glance.
                let trimmed = line.trim_start();
                if trimmed.starts_with('+') && !trimmed.starts_with("+++") {
                    theme::colored(line.clone(), Color::Green)
                } else if trimmed.starts_with('-') && !trimmed.starts_with("---") {
                    theme::colored(line.clone(), Color::Red)
                } else if trimmed.starts_with("commit ") {
                    theme::strong(line.clone(), Color::Cyan)
                } else if trimmed.starts_with("Author:") || trimmed.starts_with("Date:") {
                    theme::colored(line.clone(), Color::White)
                } else {
                    theme::plain(line.clone())
                }
            })
            .collect(),
    };
    common::render_scrollable(f, area, " Detail ", &rows, scroll, selected)
}
