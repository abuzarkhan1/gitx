use crate::views::common;
use gitx_git::models::Commit;
use ratatui::{Frame, layout::Rect};

pub fn render(
    f: &mut Frame,
    area: Rect,
    timeline: Option<&[Commit]>,
    areas: Option<&[String]>,
    scroll: usize,
    selected: usize,
) -> usize {
    let rows: Vec<String> = match timeline {
        None => vec!["No repository loaded.".to_string()],
        Some([]) => vec!["No commits found.".to_string()],
        Some(commits) => commits
            .iter()
            .enumerate()
            .map(|(i, c)| {
                // Parent-graph glyphs (docs/08 #14): `*` merge (2+ parents),
                // `•` normal, `⊙` root.
                let glyph = if c.parents.is_empty() {
                    "⊙"
                } else if c.parents.len() > 1 {
                    "*"
                } else {
                    "•"
                };
                let parents = if c.parents.is_empty() {
                    "root".to_string()
                } else {
                    c.parents
                        .iter()
                        .map(common::short_oid)
                        .collect::<Vec<_>>()
                        .join(",")
                };
                let area = areas
                    .and_then(|a| a.get(i))
                    .filter(|a| !a.is_empty())
                    .map(|a| format!("  in {a}"))
                    .unwrap_or_default();
                format!(
                    "{glyph} {}  parents {parents}{area}",
                    common::commit_line(c, 50)
                )
            })
            .collect(),
    };
    common::render_scrollable(f, area, " Commits ", &rows, scroll, selected)
}
