use crate::views::common;
use gitx_analysis::RecoveryReport;
use ratatui::{Frame, layout::Rect};

pub fn render(
    f: &mut Frame,
    area: Rect,
    recovery: Option<&RecoveryReport>,
    scroll: usize,
    selected: usize,
) -> usize {
    let rows: Vec<String> = match recovery {
        None => vec!["No repository loaded.".to_string()],
        Some(r) => {
            let mut out = vec![format!(
                "Reflog: {} entries{}  |  Unreachable commits: {}",
                r.reflog.len(),
                if r.reflog_enabled { "" } else { " (disabled)" },
                r.unreachable.len()
            )];
            for e in &r.reflog {
                let when = e.timestamp.map(common::ts).unwrap_or_else(|| "-".into());
                out.push(format!(
                    "  {:<24} {} → {}  {}  {}",
                    e.reference,
                    common::short_oid(&e.previous_oid),
                    common::short_oid(&e.new_oid),
                    when,
                    e.message
                ));
            }
            if !r.unreachable.is_empty() {
                out.push("Unreachable commits (recoverable):".to_string());
                for c in &r.unreachable {
                    out.push(format!("  {}", c.oid));
                }
            }
            if r.reflog.is_empty() && r.unreachable.is_empty() {
                out.push(
                    "Nothing to recover — reflogs empty and no unreachable commits.".to_string(),
                );
            }
            out
        }
    };
    common::render_scrollable(f, area, " Recovery (read-only) ", &rows, scroll, selected)
}
