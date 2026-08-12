use crate::views::{common, theme};
use gitx_git::models::Commit;
use ratatui::style::Modifier;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
};

/// Per-row (column, active-lane-count) computed by a lightweight lane
/// allocator: each commit continues in the lane that expects its oid; extra
/// parents open new lanes. Gives a plausible `git log --graph`-style layout.
fn graph_rows(commits: &[Commit]) -> Vec<(usize, usize)> {
    let mut lanes: Vec<String> = Vec::new(); // expected next oid per lane
    let mut rows = Vec::new();
    for c in commits {
        let id = c.id.to_string();
        let col = lanes.iter().position(|l| l == &id).unwrap_or_else(|| {
            lanes.push(id.clone());
            lanes.len() - 1
        });
        let active = lanes.len();
        if let Some(first) = c.parents.first() {
            lanes[col] = first.to_string();
        } else {
            lanes.remove(col);
        }
        for p in c.parents.iter().skip(1) {
            let pid = p.to_string();
            if !lanes.contains(&pid) {
                lanes.push(pid);
            }
        }
        rows.push((col, active));
    }
    rows
}

pub fn render(
    f: &mut Frame,
    area: Rect,
    timeline: Option<&[Commit]>,
    file_counts: Option<&[u32]>,
    scroll: usize,
    selected: usize,
) -> usize {
    let rows: Vec<Line<'static>> = match timeline {
        None => common::empty_rows("commits"),
        Some([]) => vec![theme::plain("No commits found.")],
        Some(commits) => {
            let graph = graph_rows(commits);
            let max_lanes = graph.iter().map(|(_, a)| *a).max().unwrap_or(1);
            commits
                .iter()
                .enumerate()
                .map(|(i, c)| {
                    let (col, active) = graph[i];
                    let mut g = String::new();
                    for lane in 0..max_lanes {
                        if lane == col {
                            let glyph = if c.parents.is_empty() {
                                'o'
                            } else if c.parents.len() > 1 {
                                '*'
                            } else {
                                '•'
                            };
                            g.push(glyph);
                        } else if lane < active {
                            g.push('│');
                        } else {
                            g.push(' ');
                        }
                    }
                    let count = file_counts
                        .and_then(|counts| counts.get(i))
                        .map(|n| format!("{n:>3}f"))
                        .unwrap_or_else(|| "  - ".to_string());
                    let graph_style = if c.parents.len() > 1 {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    };
                    Line::from(vec![
                        Span::styled(g, graph_style),
                        Span::raw(format!(
                            "  {}  {}  {:<20}  {:>5}  {}",
                            common::short_oid(&c.id),
                            common::ts(c.author.time),
                            c.author.name,
                            count,
                            common::one_line(&c.message, 60)
                        )),
                    ])
                })
                .collect()
        }
    };
    common::render_scrollable(
        f,
        area,
        " Timeline — commit graph (• normal · * merge · o root) ",
        &rows,
        scroll,
        selected,
    )
}
