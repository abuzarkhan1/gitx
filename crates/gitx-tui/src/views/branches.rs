use crate::views::common;
use gitx_git::models::Branch;
use ratatui::{Frame, layout::Rect};

pub fn render(
    f: &mut Frame,
    area: Rect,
    branches: Option<&[Branch]>,
    scroll: usize,
    selected: usize,
) -> usize {
    let rows: Vec<String> = match branches {
        None => vec!["No repository loaded.".to_string()],
        Some([]) => vec!["No branches found.".to_string()],
        Some(branches) => branches
            .iter()
            .map(|b| {
                let mark = if b.is_remote { "[remote]" } else { "[local] " };
                format!("{mark} {:<24} {}", b.name, common::short_oid(&b.target))
            })
            .collect(),
    };
    common::render_scrollable(f, area, " Branches ", &rows, scroll, selected)
}
