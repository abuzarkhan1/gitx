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
            let mut files: Vec<&gitx_analysis::FileAnalysis> = a.files.iter().collect();
            files.sort_by(|x, y| {
                y.risk
                    .partial_cmp(&x.risk)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            let mut out = Vec::new();
            for file in &files {
                out.push(format!(
                    "⚠ {:.0}/100  {}  {}",
                    file.risk,
                    file.classification,
                    file.path.display()
                ));
                out.push(format!(
                    "     changes {} | churn 30d {} | fixes {} | contributors {} | ownership {:.0}%",
                    file.metrics.change_frequency,
                    file.metrics.lines_added + file.metrics.lines_deleted,
                    file.metrics.bug_fix_count,
                    file.metrics.unique_contributors,
                    file.ownership_concentration
                ));
            }
            out
        }
    };
    common::render_scrollable(f, area, " Risk (evidence-backed) ", &rows, scroll, selected)
}
