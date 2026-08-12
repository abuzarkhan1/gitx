use crate::app::ArchDiff;
use crate::views::{common, theme};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
};

/// Architecture view (docs/08 §3, docs/10 §10): a structural before/after
/// comparison (HEAD vs the newest commit ≥30 days old) plus the current
/// per-module table with churn and recently-added files.
pub fn render(
    f: &mut Frame,
    area: Rect,
    arch_diff: Option<&ArchDiff>,
    analysis: Option<&gitx_analysis::RepoAnalysis>,
    loading: bool,
    scroll: usize,
    selected: usize,
) -> usize {
    let mut rows: Vec<Line<'static>> = Vec::new();

    // ── Before/after comparison ─────────────────────────────────────
    match arch_diff {
        None if loading => {
            rows.push(theme::plain("Loading architecture analysis…"));
            rows.push(theme::dim(
                "  Background load in progress — the panel fills in when ready.",
            ));
        }
        None => {
            rows.push(theme::plain("No structural comparison available."));
            rows.push(theme::dim(
                "  Need at least two commits ≥30 days apart in history.",
            ));
        }
        Some(d) => {
            rows.push(theme::heading("Structure: before vs after"));
            rows.push(theme::kv("Before", d.from.clone(), Color::Yellow));
            rows.push(theme::kv("After", d.to.clone(), Color::Green));
            rows.push(Line::default());
            rows.push(theme::kv(
                "Files added",
                format!("{}", d.added),
                Color::Green,
            ));
            rows.push(theme::kv(
                "Files removed",
                format!("{}", d.removed),
                Color::Red,
            ));
            rows.push(theme::kv(
                "Files modified",
                format!("{}", d.modified),
                Color::Cyan,
            ));
            if !d.modules_added.is_empty() {
                rows.push(theme::heading("Modules added"));
                for m in d.modules_added.iter().take(8) {
                    rows.push(theme::plain(format!("  + {m}/")));
                }
            }
            if !d.added_files.is_empty() {
                rows.push(theme::heading("Added files (top)"));
                for path in d.added_files.iter().take(8) {
                    rows.push(theme::colored(format!("  + {}", path), Color::Green));
                }
            }
            if !d.removed_files.is_empty() {
                rows.push(theme::heading("Removed files (top)"));
                for path in d.removed_files.iter().take(8) {
                    rows.push(theme::colored(format!("  - {}", path), Color::Red));
                }
            }
        }
    }

    // ── Current module table ────────────────────────────────────────
    rows.push(Line::default());
    rows.push(theme::heading("Current modules — size + churn + new files"));
    if let Some(a) = analysis
        && !a.files.is_empty()
    {
        let cutoff = chrono::Utc::now().timestamp() - 90 * 86_400;
        let mut dirs: std::collections::HashMap<String, (usize, u64, usize)> =
            std::collections::HashMap::new();
        for file in &a.files {
            let dir = file
                .path
                .parent()
                .filter(|p| !p.as_os_str().is_empty())
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| ".".into());
            let entry = dirs.entry(dir).or_insert((0, 0, 0));
            entry.0 += 1;
            entry.1 += file.metrics.lines_added as u64 + file.metrics.lines_deleted as u64;
            if file
                .metrics
                .first_introduced
                .map(|t| t.timestamp() >= cutoff)
                .unwrap_or(false)
            {
                entry.2 += 1;
            }
        }
        let mut list: Vec<(String, usize, u64, usize)> = dirs
            .into_iter()
            .map(|(dir, (files, churn, recent))| (dir, files, churn, recent))
            .collect();
        list.sort_by_key(|(_, files, _, _)| std::cmp::Reverse(*files));
        let max_files = list
            .iter()
            .map(|(_, files, _, _)| *files)
            .max()
            .unwrap_or(1)
            .max(1);
        for (dir, files, churn, recent) in list.iter().take(20) {
            let width = 20usize;
            let filled = (files * width).div_ceil(max_files);
            let bar: String = "█".repeat(filled) + &"░".repeat(width - filled);
            let recent_mark = if *recent > 0 {
                format!("  +{recent} new(90d)")
            } else {
                String::new()
            };
            rows.push(Line::from(vec![
                Span::styled(format!("{dir:<36} "), Style::default().fg(Color::White)),
                Span::styled(bar, Style::default().fg(Color::Cyan)),
                Span::styled(
                    format!(" {files:>3} files {churn:>7} churn{recent_mark}"),
                    Style::default().fg(Color::DarkGray),
                ),
            ]));
        }
    } else {
        rows.push(theme::dim(
            "  (no analysis — run `gitx refresh` or press r)",
        ));
    }

    common::render_scrollable(
        f,
        area,
        " Architecture — structural before/after + modules ",
        &rows,
        scroll,
        selected,
    )
}
