use crate::views::common;
use ratatui::{Frame, layout::Rect};

pub fn render(
    f: &mut Frame,
    area: Rect,
    text: Option<&[String]>,
    scroll: usize,
    selected: usize,
) -> usize {
    let rows: Vec<String> = match text {
        None => vec!["Nothing selected.".to_string()],
        Some([]) => vec!["Nothing selected.".to_string()],
        Some(lines) => lines.to_vec(),
    };
    common::render_scrollable(f, area, " Detail ", &rows, scroll, selected)
}
