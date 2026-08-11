use crate::views::common;
use gitx_git::models::Commit;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};

pub fn render(
    f: &mut Frame,
    area: Rect,
    query: &str,
    results: Option<&[Commit]>,
    focused: bool,
    scroll: usize,
    selected: usize,
) -> usize {
    let border_style = if focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };
    let block = Block::default()
        .title(" Search ")
        .borders(Borders::ALL)
        .style(border_style);
    let area = block.inner(area);
    f.render_widget(block, area);

    // Query input row.
    let input_area = Rect::new(area.x, area.y, area.width, 1);
    let input = Paragraph::new(format!("> {}", query));
    f.render_widget(input, input_area);

    let results_area = Rect::new(
        area.x,
        area.y + 2,
        area.width,
        area.height.saturating_sub(2),
    );
    let rows: Vec<String> = if query.trim().is_empty() {
        vec!["Type to search commit messages, authors, and oids (in-memory over the loaded timeline).".to_string()]
    } else {
        match results {
            None => vec!["No results.".to_string()],
            Some([]) => vec!["No results.".to_string()],
            Some(list) => list.iter().map(|c| common::commit_line(c, 90)).collect(),
        }
    };
    common::render_scrollable(f, results_area, "", &rows, scroll, selected)
}
