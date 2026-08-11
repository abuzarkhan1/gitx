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
        Some(commits) => commits.iter().map(|c| common::commit_line(c, 70)).collect(),
    };
    common::render_scrollable(f, area, " Timeline ", &rows, scroll, selected)
}
