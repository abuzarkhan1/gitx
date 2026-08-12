use crate::views::{common, theme};
use gitx_git::models::Commit;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

pub fn render(
    f: &mut Frame,
    area: Rect,
    timeline: Option<&[Commit]>,
    areas: Option<&[String]>,
    commit_files: Option<&[Vec<String>]>,
    scroll: usize,
    selected: usize,
) -> usize {
    let rows: Vec<Line<'static>> = match timeline {
        None => common::empty_rows("commits"),
        Some([]) => vec![theme::plain("No commits found.")],
        Some(commits) => {
            let mut out: Vec<Line<'static>> = Vec::new();
            for (i, c) in commits.iter().enumerate() {
                // Parent-graph glyphs (docs/08 #14): `*` merge (2+ parents),
                // `•` normal, `⊙` root — colored by kind.
                let (glyph, gcolor) = if c.parents.is_empty() {
                    ("⊙", Color::Magenta)
                } else if c.parents.len() > 1 {
                    ("*", Color::Yellow)
                } else {
                    ("•", Color::Cyan)
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
                out.push(Line::from(vec![
                    Span::styled(
                        glyph,
                        Style::default().fg(gcolor).add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("  "),
                    common::commit_line(c, 50),
                    Span::styled(
                        format!("  parents {parents}{area}"),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]));
                // Related-commits panel (docs/08 #23): for the selected row,
                // list commits touching overlapping files, by overlap size.
                if i == selected {
                    if let Some(related) = related_commits(c, commits, commit_files, i) {
                        for (oid, overlap) in related {
                            out.push(theme::dim(format!(
                                "      ↳ related {oid}  ({overlap} shared file{})",
                                if overlap == 1 { "" } else { "s" }
                            )));
                        }
                    } else {
                        out.push(theme::dim("      ↳ (no shared files — nothing to compare)"));
                    }
                }
            }
            out
        }
    };
    common::render_scrollable(
        f,
        area,
        " Commits — parents + affected areas + related (Enter: full detail) ",
        &rows,
        scroll,
        selected,
    )
}

/// Commits (other than `selected`) whose changed-file set intersects the
/// selected commit's, ranked by overlap size. Uses the per-commit file sets
/// computed at load time; when unavailable (cache path) returns None.
fn related_commits(
    selected: &Commit,
    commits: &[Commit],
    commit_files: Option<&[Vec<String>]>,
    index: usize,
) -> Option<Vec<(String, usize)>> {
    let files = commit_files?.get(index)?;
    if files.is_empty() {
        return None;
    }
    let selected_set: std::collections::HashSet<&String> = files.iter().collect();
    let mut related: Vec<(String, usize)> = Vec::new();
    for (i, c) in commits.iter().enumerate() {
        if i == index || c.id == selected.id {
            continue;
        }
        let Some(other) = commit_files?.get(i) else {
            continue;
        };
        let overlap = other.iter().filter(|f| selected_set.contains(f)).count();
        if overlap > 0 {
            related.push((common::short_oid(&c.id), overlap));
        }
    }
    related.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
    Some(related.into_iter().take(3).collect())
}
