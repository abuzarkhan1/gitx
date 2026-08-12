use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};

pub fn render(
    f: &mut Frame,
    area: Rect,
    query: &str,
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

    // Query input row.
    let input_area = Rect::new(inner.x, inner.y, inner.width, 1);
    let input = Paragraph::new(format!("> {}", query));
    f.render_widget(input, input_area);

    let results_area = Rect::new(
        inner.x,
        inner.y + 2,
        inner.width,
        inner.height.saturating_sub(2),
    );
    let rows: Vec<String> = if query.trim().is_empty() {
        vec![
            "Type to search across commits, files, authors, branches and tags \
             (SQLite FTS5 over the index)."
                .to_string(),
        ]
    } else {
        match results {
            None => vec!["Searching…".to_string()],
            Some([]) => vec!["No results.".to_string()],
            Some(list) => list
                .iter()
                .map(|hit| {
                    let badge = match hit.scope.as_str() {
                        "commit" => "commit ",
                        "file" => "file   ",
                        "author" => "author ",
                        "branch" => "branch ",
                        "tag" => "tag    ",
                        _ => "       ",
                    };
                    let detail = if hit.detail.is_empty() {
                        String::new()
                    } else {
                        format!("  ({})", hit.detail)
                    };
                    format!("{badge} {}  {}{detail}", hit.id, hit.title)
                })
                .collect(),
        }
    };

    let mut text = String::new();
    for (i, row) in rows.iter().enumerate() {
        if i == selected && focused {
            text.push_str(&format!("\u{1b}[7m{row}\u{1b}[0m\n"));
        } else {
            text.push_str(&format!("{row}\n"));
        }
    }
    let paragraph = Paragraph::new(text).scroll((scroll as u16, 0));
    f.render_widget(paragraph, results_area);
    rows.len()
}
