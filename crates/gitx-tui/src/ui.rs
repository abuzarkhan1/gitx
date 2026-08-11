use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

use crate::app::{App, View};
use crate::views;

pub fn render(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Main content
            Constraint::Length(1), // Status bar
        ])
        .split(f.area());

    render_header(f, chunks[0]);
    render_main(f, app, chunks[1]);
    render_status_bar(f, app, chunks[2]);
}

fn render_header(f: &mut Frame, area: Rect) {
    let header = Paragraph::new(" GitX   Repository   Branch   State ")
        .block(Block::default().borders(Borders::ALL))
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(header, area);
}

fn render_main(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(20), // Navigation sidebar
            Constraint::Min(0),     // Content area
        ])
        .split(area);

    render_navigation(f, app, chunks[0]);
    render_content(f, app, chunks[1]);
}

fn render_navigation(f: &mut Frame, app: &mut App, area: Rect) {
    let views = vec![
        "Overview",
        "Timeline",
        "Commits",
        "Branches",
        "Files",
        "Contributors",
        "Hotspots",
        "Ownership",
        "Architecture",
        "Dependencies",
        "Risk",
        "Health",
        "Recovery",
        "Search",
    ];

    let items: Vec<ListItem> = views.into_iter().map(ListItem::new).collect();

    let mut state = ListState::default();
    state.select(Some(app.nav_index));

    let list = List::new(items)
        .block(Block::default().title(" Navigation ").borders(Borders::ALL))
        .highlight_style(Style::default().bg(Color::DarkGray).fg(Color::White))
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, area, &mut state);
}

fn render_content(f: &mut Frame, app: &mut App, area: Rect) {
    // Each render returns the number of scrollable rows so scroll/selection
    // can be clamped (docs/08: j/k scrolls, Enter opens).
    let row_count = match app.current_view {
        View::Overview => {
            views::overview::render(f, area, app.stats.as_ref());
            0
        }
        View::Timeline => {
            views::timeline::render(f, area, app.timeline.as_deref(), app.scroll, app.selected)
        }
        View::Commits => {
            views::commits::render(f, area, app.timeline.as_deref(), app.scroll, app.selected)
        }
        View::Branches => {
            views::branches::render(f, area, app.branches.as_deref(), app.scroll, app.selected)
        }
        View::Files => {
            views::files::render(f, area, app.hotspots.as_ref(), app.scroll, app.selected)
        }
        View::Contributors => views::contributors::render(
            f,
            area,
            app.contributors.as_deref(),
            app.scroll,
            app.selected,
        ),
        View::Hotspots => {
            views::hotspots::render(f, area, app.hotspots.as_ref(), app.scroll, app.selected)
        }
        View::Ownership => {
            views::ownership::render(f, area, app.hotspots.as_ref(), app.scroll, app.selected)
        }
        View::Architecture => {
            // Architecture panel (docs/08): directory/module evolution from the
            // analyzed file set.
            views::overview::architecture_panel(f, area, app.hotspots.as_ref());
            0
        }
        View::Dependencies => views::dependencies::render(
            f,
            area,
            app.dependencies.as_deref(),
            app.scroll,
            app.selected,
        ),
        View::Risk => views::risk::render(f, area, app.hotspots.as_ref(), app.scroll, app.selected),
        View::Health => {
            views::overview::health_panel(f, area, app.hotspots.as_ref());
            0
        }
        View::Recovery => {
            views::recovery::render(f, area, app.recovery.as_ref(), app.scroll, app.selected)
        }
        View::Search => views::search::render(
            f,
            area,
            &app.search_query,
            app.search_results.as_deref(),
            app.in_content,
            app.scroll,
            app.selected,
        ),
        View::Detail => views::detail::render(
            f,
            area,
            app.detail_text.as_deref(),
            app.detail_scroll,
            app.selected,
        ),
    };
    if app.selected >= row_count && row_count > 0 {
        app.selected = row_count - 1;
    }

    if let Some(ref err) = app.error {
        let err_rect = centered_rect(60, 20, area);
        let err_p = Paragraph::new(err.as_str())
            .block(Block::default().title(" Error ").borders(Borders::ALL))
            .style(Style::default().fg(Color::Red));
        f.render_widget(err_p, err_rect);
    }
}

fn render_status_bar(f: &mut Frame, app: &mut App, area: Rect) {
    let status = if app.loading {
        " Loading... | Press 'q' to quit "
    } else if app.current_view == crate::app::View::Detail {
        " j/k Scroll  Enter Next  Esc/← Close detail  q Quit "
    } else if app.in_content {
        " j/k Scroll  Enter Open  Esc/← Back to navigation  / Search  q Quit "
    } else {
        " ↑↓ Navigate  Enter Open  / Search  ? Help  q Quit "
    };

    let p = Paragraph::new(status).style(Style::default().bg(Color::Blue).fg(Color::White));
    f.render_widget(p, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
