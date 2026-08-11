use crate::views::common;
use gitx_git::models::Commit;
use ratatui::{Frame, layout::Rect};

pub fn render(
    f: &mut Frame,
    area: Rect,
    timeline: Option<&[Commit]>,
    scroll: usize,
    selected: usize,
) -> usize {
    let rows: Vec<String> = match timeline {
        None => vec!["No repository loaded.".to_string()],
        Some([]) => vec!["No commits found.".to_string()],
        Some(commits) => commits
            .iter()
            .map(|c| {
                let parents = if c.parents.is_empty() {
                    "root".to_string()
                } else {
                    c.parents
                        .iter()
                        .map(common::short_oid)
                        .collect::<Vec<_>>()
                        .join(",")
                };
                format!("{}  parents {}", common::commit_line(c, 60), parents)
            })
            .collect(),
    };
    common::render_scrollable(f, area, " Commits ", &rows, scroll, selected)
}
