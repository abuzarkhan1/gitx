use crate::views::common;
use gitx_analysis::RepoAnalysis;
use ratatui::{Frame, layout::Rect};

pub fn render(
    f: &mut Frame,
    area: Rect,
    analysis: Option<&RepoAnalysis>,
    scroll: usize,
    selected: usize,
) -> usize {
    let rows: Vec<String> = match analysis {
        None => vec!["No repository loaded.".to_string()],
        Some(a) if a.files.is_empty() => vec!["No files analyzed.".to_string()],
        Some(a) => {
            let mut out = Vec::new();
            for file in &a.files {
                out.push(format!(
                    "{:>5.0}  {:<8}  {}",
                    file.hotspot,
                    file.classification,
                    file.path.display()
                ));
                out.push(format!(
                    "         changes {} | churn {} | fixes {} | contributors {} | ownership {:.0}%",
                    file.metrics.change_frequency,
                    file.metrics.lines_added + file.metrics.lines_deleted,
                    file.metrics.bug_fix_count,
                    file.metrics.unique_contributors,
                    file.ownership_concentration,
                ));
            }
            out.push(format!(
                "\n{} files analyzed, {} commits",
                a.files.len(),
                a.total_commits
            ));
            out
        }
    };
    common::render_scrollable(
        f,
        area,
        " Hotspots (change/maintenance risk, 0–100) ",
        &rows,
        scroll,
        selected,
    )
}
