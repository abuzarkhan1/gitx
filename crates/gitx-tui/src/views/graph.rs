//! Graph view (docs/21 Stage 6, docs/02 V1 "stronger architecture graph"):
//! per-directory module summary from the shared HEAD-graph builder — file
//! count, import edges and call edges per directory.

use crate::views::{common, theme};
use ratatui::{Frame, layout::Rect, style::Color, text::Line};

/// Render the module table. Returns the number of rows for the cursor.
pub fn render(
    f: &mut Frame,
    area: Rect,
    summary: Option<&[(String, usize, usize, usize)]>,
    scroll: usize,
    selected: usize,
) -> usize {
    let mut rows: Vec<Line<'static>> = Vec::new();
    rows.push(theme::heading(
        "Modules — files, import & call edges per directory",
    ));
    rows.push(theme::dim(
        "  heuristic extraction from the HEAD tree (docs/21 Stage 6)",
    ));

    match summary {
        None => {
            rows.push(Line::default());
            rows.push(theme::plain("No graph data available."));
            rows.push(theme::dim("  Run: gitx refresh"));
        }
        Some(list) if list.is_empty() => {
            rows.push(Line::default());
            rows.push(theme::plain("No source modules found in HEAD."));
        }
        Some(list) => {
            rows.push(Line::default());
            let mut total_files = 0usize;
            let mut total_imports = 0usize;
            let mut total_calls = 0usize;
            for (dir, files, imports, calls) in list {
                total_files += files;
                total_imports += imports;
                total_calls += calls;
                let mut label = dir.clone();
                if label.chars().count() > 42 {
                    label = label.chars().take(41).collect::<String>() + "…";
                }
                rows.push(theme::plain(format!(
                    "{label:<42} {:>6} files  {:>6} imports  {:>6} calls",
                    files, imports, calls
                )));
            }
            rows.push(Line::default());
            rows.push(theme::kv(
                "Totals",
                format!(
                    "{total_files} files, {total_imports} import edges, {total_calls} call edges",
                ),
                Color::Yellow,
            ));
        }
    }

    common::render_scrollable(f, area, " Graph ", &rows, scroll, selected)
}
