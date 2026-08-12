use crate::views::{common, theme};
use gitx_analysis::RepoAnalysis;
use ratatui::{Frame, layout::Rect, text::Line};

pub fn render(
    f: &mut Frame,
    area: Rect,
    analysis: Option<&RepoAnalysis>,
    sort_mode: u8,
    scroll: usize,
    selected: usize,
) -> usize {
    let rows: Vec<Line<'static>> = match analysis {
        None => common::empty_rows("repository analysis"),
        Some(a) if a.files.is_empty() => vec![theme::plain("No files analyzed.")],
        Some(a) => {
            let mut idx: Vec<usize> = (0..a.files.len()).collect();
            let key = |i: usize| match sort_mode {
                1 => a.files[i].metrics.change_frequency as i64,
                2 => {
                    a.files[i].metrics.lines_added as i64 + a.files[i].metrics.lines_deleted as i64
                }
                _ => (a.files[i].hotspot * 1000.0) as i64,
            };
            idx.sort_by_key(|i| std::cmp::Reverse(key(*i)));
            let mut out = Vec::new();
            for i in idx {
                let file = &a.files[i];
                let churn = file.metrics.lines_added + file.metrics.lines_deleted;
                out.push(theme::hbar(
                    file.path.display().to_string(),
                    file.hotspot,
                    24,
                    theme::severity_color(file.hotspot),
                ));
                out.push(theme::dim(format!(
                    "         {}  changes {} · churn {} · fixes {} · contributors {} · ownership {:.0}%",
                    file.classification,
                    file.metrics.change_frequency,
                    churn,
                    file.metrics.bug_fix_count,
                    file.metrics.unique_contributors,
                    file.ownership_concentration
                )));
            }
            out.push(Line::default());
            out.push(theme::dim(format!(
                "{} files analyzed · {} commits · sort: {} (press s to change)",
                a.files.len(),
                a.total_commits,
                match sort_mode {
                    1 => "changes",
                    2 => "churn",
                    _ => "score",
                }
            )));
            out
        }
    };
    common::render_scrollable(
        f,
        area,
        " Hotspots — files ranked by maintenance risk (0–100) ",
        &rows,
        scroll,
        selected,
    )
}
