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
    loading: bool,
    scroll: usize,
    selected: usize,
) -> usize {
    let rows: Vec<Line<'static>> = match recovery {
        None => common::panel_placeholder(loading, "recovery report"),
        Some(r) => {
            let mut out = vec![theme::strong(
                format!(
                    "Reflog: {} entries{}  |  Unreachable commits: {}  |  Dangling objects: {}",
                    r.reflog.len(),
                    if r.reflog_enabled { "" } else { " (disabled)" },
                    r.unreachable.len(),
                    r.dangling.len()
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
            // Dangling trees/blobs (docs/08 §3 Recovery, docs/12 §6): objects
            // present in the object database but not reachable from any ref.
            // The header line already counts them via the count in the title.
            if !r.dangling.is_empty() {
                out.push(theme::strong(
                    format!(
                        "Dangling objects ({} trees, {} blobs):",
                        r.dangling
                            .iter()
                            .filter(|d| matches!(d.kind, gitx_analysis::DanglingKind::Tree))
                            .count(),
                        r.dangling
                            .iter()
                            .filter(|d| matches!(d.kind, gitx_analysis::DanglingKind::Blob))
                            .count()
                    ),
                    Color::Magenta,
                ));
                for d in r.dangling.iter().take(200) {
                    out.push(Line::from(vec![
                        Span::styled(
                            format!("  {:<5} ", d.kind),
                            Style::default().fg(Color::Magenta),
                        ),
                        Span::styled(d.oid.to_string(), Style::default().fg(Color::DarkGray)),
                    ]));
                }
            }
            if r.reflog.is_empty() && r.unreachable.is_empty() && r.dangling.is_empty() {
                out.push(theme::plain(
                    "Nothing to recover — reflogs empty, no unreachable commits, no dangling objects.",
                ));
            }
            out
        }
    };
    common::render_scrollable(f, area, " Recovery (read-only) ", &rows, scroll, selected)
}
