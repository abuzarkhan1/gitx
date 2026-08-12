use crate::views::{common, theme};
use gitx_analysis::RecoveryReport;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
};

pub fn render(
    f: &mut Frame,
    area: Rect,
    recovery: Option<&RecoveryReport>,
    scroll: usize,
    selected: usize,
) -> usize {
    let rows: Vec<Line<'static>> = match recovery {
        None => common::empty_rows("repository"),
        Some(r) => {
            let mut out = vec![theme::strong(
                format!(
                    "Reflog: {} entries{}  |  Unreachable commits: {}",
                    r.reflog.len(),
                    if r.reflog_enabled { "" } else { " (disabled)" },
                    r.unreachable.len()
                ),
                Color::Cyan,
            )];
            for e in &r.reflog {
                let when = e.timestamp.map(common::ts).unwrap_or_else(|| "-".into());
                out.push(Line::from(vec![
                    Span::raw(format!("  {:<24} ", e.reference)),
                    Span::styled(
                        format!(
                            "{} → {}",
                            common::short_oid(&e.previous_oid),
                            common::short_oid(&e.new_oid)
                        ),
                        Style::default().fg(Color::Yellow),
                    ),
                    Span::styled(
                        format!("  {when}  {}", e.message),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]));
            }
            if !r.unreachable.is_empty() {
                out.push(theme::strong(
                    "Unreachable commits (recoverable):",
                    Color::Yellow,
                ));
                for c in &r.unreachable {
                    out.push(theme::plain(format!("  {}", c.oid)));
                }
            }
            if r.reflog.is_empty() && r.unreachable.is_empty() {
                out.push(theme::plain(
                    "Nothing to recover — reflogs empty and no unreachable commits.",
                ));
            }
            out
        }
    };
    common::render_scrollable(f, area, " Recovery (read-only) ", &rows, scroll, selected)
}
