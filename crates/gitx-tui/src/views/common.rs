use gitx_git::models::Commit;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};

/// Render a titled, scrollable-free text panel with a fallback message.
pub fn panel<'a>(title: &'a str, content: String, color: Color) -> Paragraph<'a> {
    Paragraph::new(content)
        .block(
            Block::default()
                .title(title.to_string())
                .borders(Borders::ALL)
                .style(Style::default().fg(color)),
        )
        .wrap(Wrap { trim: false })
        .style(Style::default().fg(Color::White))
}

pub fn render(f: &mut Frame, area: Rect, title: &str, content: String) {
    f.render_widget(panel(title, content, Color::White), area);
}

pub fn short_oid(id: &gitx_git::models::ObjectId) -> String {
    id.to_string().chars().take(7).collect()
}

pub fn ts(seconds: i64) -> String {
    match chrono::DateTime::from_timestamp(seconds, 0) {
        Some(dt) => dt.format("%Y-%m-%d %H:%M").to_string(),
        None => seconds.to_string(),
    }
}

pub fn one_line(message: &str, max: usize) -> String {
    let first = message.lines().next().unwrap_or("");
    first.chars().take(max).collect()
}

pub fn commit_line(c: &Commit, max_message: usize) -> String {
    format!(
        "{}  {}  {:<20}  {}",
        short_oid(&c.id),
        ts(c.author.time),
        c.author.name,
        one_line(&c.message, max_message)
    )
}

/// Render a titled list with keyboard scrolling (docs/08: j/k scrolls, the
/// selected row is highlighted). `rows` are the full content lines; `scroll`
/// is the first visible row. Returns the number of rows so the caller can
/// clamp scroll offsets.
pub fn render_scrollable(
    f: &mut Frame,
    area: Rect,
    title: &str,
    rows: &[String],
    scroll: usize,
    selected: usize,
) -> usize {
    let visible = area.height.saturating_sub(2) as usize;
    let scroll = scroll.min(rows.len().saturating_sub(visible));
    let end = (scroll + visible).min(rows.len());

    let items: Vec<ListItem> = rows[scroll..end]
        .iter()
        .enumerate()
        .map(|(i, line)| {
            let style = if scroll + i == selected {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(line.clone()).style(style)
        })
        .collect();

    let mut state = ListState::default();
    if selected >= scroll && selected < end {
        state.select(Some(selected - scroll));
    }
    let list = List::new(items)
        .block(
            Block::default()
                .title(title.to_string())
                .borders(Borders::ALL),
        )
        .highlight_symbol("> ");
    f.render_stateful_widget(list, area, &mut state);
    rows.len()
}
