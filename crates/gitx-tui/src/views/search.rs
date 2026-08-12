use crate::views::theme;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListState, Paragraph},
};

#[allow(clippy::too_many_arguments)]
pub fn render(
    f: &mut Frame,
    area: Rect,
    query: &str,
    pending: bool,
    results: Option<&[gitx_services::SearchHit]>,
    focused: bool,
    scroll: usize,
    selected: usize,
) -> usize {
    let border_style = if focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };
    let block = Block::default()
        .title(" Search (FTS: commits · files · authors · branches · tags) ")
        .borders(Borders::ALL)
        .style(border_style);
    let inner = block.inner(area);
    f.render_widget(block, area);

    // Query input row with a visible blinking cursor (docs/08 #20).
    let input_area = Rect::new(inner.x, inner.y, inner.width, 1);
    let input = Paragraph::new(Line::from(vec![
        Span::styled(
            "> ",
            Style::default()
                .fg(theme::global().accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(query.to_string()),
        if focused {
            Span::styled("▌", Style::default().fg(Color::White))
        } else {
            Span::raw(" ")
        },
    ]));
    f.render_widget(input, input_area);

    let results_area = Rect::new(
        inner.x,
        inner.y + 2,
        inner.width,
        inner.height.saturating_sub(2),
    );
    let rows: Vec<Line<'static>> = if query.trim().is_empty() {
        vec![
            theme::plain("Type to search across commits, files, authors, branches and tags"),
            theme::dim("(SQLite FTS5 over the index — Enter opens the selected result.)"),
        ]
    } else if pending {
        vec![theme::dim("Searching the index…")]
    } else {
        match results {
            None => vec![theme::dim("Type more to search…")],
            Some([]) => vec![
                theme::plain("No results."),
                theme::dim("Try a shorter term, or run `gitx refresh` to rebuild the index."),
            ],
            Some(list) => {
                let mut out = Vec::new();
                for (i, hit) in list.iter().enumerate() {
                    let (badge, color) = match hit.scope.as_str() {
                        "commit" => ("commit ", Color::Cyan),
                        "file" => ("file   ", Color::Green),
                        "author" => ("author ", Color::Magenta),
                        "branch" => ("branch ", Color::Yellow),
                        "tag" => ("tag    ", Color::Blue),
                        _ => ("       ", Color::DarkGray),
                    };
                    let detail = if hit.detail.is_empty() {
                        String::new()
                    } else {
                        format!("  ({})", hit.detail)
                    };
                    let line = Line::from(vec![
                        Span::styled(
                            badge,
                            Style::default().fg(color).add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(hit.id.clone()),
                        Span::raw("  "),
                        Span::styled(hit.title.clone(), Style::default().fg(Color::White)),
                        Span::styled(detail, Style::default().fg(Color::DarkGray)),
                    ]);
                    let _ = i;
                    out.push(line);
                }
                out
            }
        }
    };

    let total = rows.len();
    let visible = results_area.height.saturating_sub(2) as usize;
    let scroll = scroll.min(total.saturating_sub(visible));
    let end = (scroll + visible).min(total);
    let mut state = ListState::default();
    if selected >= scroll && selected < end {
        state.select(Some(selected - scroll));
    }
    let list = List::new(rows[scroll..end].to_vec())
        .highlight_style(
            Style::default()
                .bg(theme::global().sel_bg)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");
    f.render_stateful_widget(list, results_area, &mut state);
    total
}
