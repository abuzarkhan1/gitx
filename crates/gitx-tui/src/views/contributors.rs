use crate::views::common;
use ratatui::{Frame, layout::Rect};

pub fn render(
    f: &mut Frame,
    area: Rect,
    contributors: Option<&[(String, u64)]>,
    scroll: usize,
    selected: usize,
) -> usize {
    let rows: Vec<String> = match contributors {
        None => vec!["No repository loaded.".to_string()],
        Some([]) => vec!["No contributors found.".to_string()],
        Some(list) => list
            .iter()
            .map(|(key, count)| format!("{:>6} commits  {}", count, key))
            .collect(),
    };
    common::render_scrollable(f, area, " Contributors ", &rows, scroll, selected)
}
