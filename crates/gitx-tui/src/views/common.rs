use gitx_git::models::Commit;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};

use super::theme::{self, Theme};

/// Render a titled, scrollable-free text panel with a fallback message.
pub fn panel<'a>(title: &'a str, content: String, color: ratatui::style::Color) -> Paragraph<'a> {
    Paragraph::new(content)
        .block(
            Block::default()
                .title(title.to_string())
                .borders(Borders::ALL)
                .style(Style::default().fg(color)),
        )
        .wrap(Wrap { trim: false })
        .style(Style::default().fg(theme::global().fg))
}

pub fn render(f: &mut Frame, area: Rect, title: &str, content: String) {
    f.render_widget(panel(title, content, ratatui::style::Color::White), area);
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

pub fn commit_line(c: &Commit, max_message: usize) -> Span<'static> {
    Span::raw(format!(
        "{}  {}  {:<20}  {}",
        short_oid(&c.id),
        ts(c.author.time),
        c.author.name,
        one_line(&c.message, max_message)
    ))
}

/// Empty-state guidance rows (docs/08 §7): a message plus the documented
/// recovery action, consistently across every view.
pub fn empty_rows(kind: &str) -> Vec<Line<'static>> {
    vec![
        theme::plain(format!("No {kind} available.")),
        theme::dim("Run: gitx refresh to build the repository index."),
    ]
}

/// A dimmed one-line description of what the current view shows (docs/25
/// evidence-first output: the user always knows what they are looking at).
pub fn desc(text: &str) -> Line<'static> {
    theme::dim(text)
}

/// Render a titled list with keyboard scrolling (docs/08: j/k scrolls, the
/// selected row is highlighted with the theme's selection color). `rows` are
/// the full content lines; `scroll` is the first visible row. Returns the
/// number of rows so the caller can clamp scroll offsets.
pub fn render_scrollable(
    f: &mut Frame,
    area: Rect,
    title: &str,
    rows: &[Line<'static>],
    scroll: usize,
    selected: usize,
) -> usize {
    let visible = area.height.saturating_sub(2) as usize;
    let scroll = scroll.min(rows.len().saturating_sub(visible));
    let end = (scroll + visible).min(rows.len());
    let theme: &Theme = theme::global();

    let items: Vec<ListItem> = rows[scroll..end]
        .iter()
        .enumerate()
        .map(|(i, line)| {
            let style = if scroll + i == selected {
                Style::default()
                    .bg(theme.sel_bg)
                    .fg(theme.fg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.fg)
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
        .highlight_symbol("▶ ");
    f.render_stateful_widget(list, area, &mut state);
    rows.len()
}
