use crate::views::common;
use gitx_git::models::Commit;
use ratatui::{Frame, layout::Rect};

pub fn render(
    f: &mut Frame,
    area: Rect,
    timeline: Option<&[Commit]>,
    file_counts: Option<&[u32]>,
    scroll: usize,
    selected: usize,
) -> usize {
    let rows: Vec<String> = match timeline {
        None => common::empty_rows("commits"),
        Some([]) => vec!["No commits found.".to_string()],
        Some(commits) => commits
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let count = file_counts
                    .and_then(|counts| counts.get(i))
                    .map(|n| format!("{n:>3}f"))
                    .unwrap_or_else(|| "  - ".to_string());
                format!(
                    "{}  {}  {:<20}  {:>5}  {}",
                    common::short_oid(&c.id),
                    common::ts(c.author.time),
                    c.author.name,
                    count,
                    common::one_line(&c.message, 70)
                )
            })
            .collect(),
    };
    common::render_scrollable(f, area, " Timeline ", &rows, scroll, selected)
}
