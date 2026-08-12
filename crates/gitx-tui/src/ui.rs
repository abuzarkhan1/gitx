use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

use crate::app::{App, View};
use crate::views;

/// Minimum usable terminal size (docs/08 §6 responsive small-terminal layout).
/// Below this the layout collapses; show guidance instead of broken panels.
pub const MIN_WIDTH: u16 = 60;
pub const MIN_HEIGHT: u16 = 20;

pub fn render(f: &mut Frame, app: &mut App) {
    let area = f.area();
    if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
        render_too_small(f, area);
        return;
    }
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Main content
            Constraint::Length(1), // Status bar
        ])
        .split(area);

    render_header(f, app, chunks[0]);
    render_main(f, app, chunks[1]);
    render_status_bar(f, app, chunks[2]);

    if app.show_help {
        render_help(f, chunks[1]);
    }
}

/// Terminal-too-small guidance (docs/08 §6): centered message with the
/// required minimum size instead of a mangled layout.
fn render_too_small(f: &mut Frame, area: Rect) {
    let msg = format!(
        " Terminal too small — need at least {MIN_WIDTH}×{MIN_HEIGHT} (have {}×{}) ",
        area.width, area.height
    );
    let rect = centered_rect(70, 20, area);
    let p = Paragraph::new(msg)
        .block(Block::default().title(" GitX ").borders(Borders::ALL))
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(p, rect);
}

fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let name = app
        .repo_path
        .as_deref()
        .map(|p| {
            std::path::Path::new(p)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| p.to_string())
        })
        .unwrap_or_else(|| "no repository".to_string());
    let branch = app
        .branches
        .as_ref()
        .and_then(|b| b.iter().find(|b| !b.is_remote))
        .map(|b| b.name.clone())
        .unwrap_or_else(|| "-".into());
    let state = app.repo_state.as_deref().unwrap_or("clean").to_lowercase();

    let header = Paragraph::new(format!(" GitX   {name}   {branch}   {state} "))
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
        View::Overview => views::overview::render(f, area, app),
        View::Timeline => views::timeline::render(
            f,
            area,
            app.timeline.as_deref(),
            app.timeline_file_counts.as_deref(),
            app.scroll,
            app.selected,
        ),
        View::Commits => views::commits::render(
            f,
            area,
            app.timeline.as_deref(),
            app.timeline_areas.as_deref(),
            app.scroll,
            app.selected,
        ),
        View::Branches => views::branches::render(
            f,
            area,
            app.branches.as_deref(),
            app.branch_tips.as_deref(),
            app.scroll,
            app.selected,
        ),
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
        View::Health => views::health::render(f, area, app),
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
        " Reloading... | Press 'q' to quit "
    } else if app.show_help {
        " Esc Close help  q Quit "
    } else if app.current_view == crate::app::View::Detail {
        " j/k Scroll  Enter Next  Esc/← Close detail  q Quit "
    } else if app.in_content {
        " j/k Scroll  Enter Open  Esc/← Back to navigation  / Search  r Refresh  q Quit "
    } else {
        " ↑↓ Navigate  Enter Open  / Search  ? Help  r Refresh  q Quit "
    };

    let p = Paragraph::new(status).style(Style::default().bg(Color::Blue).fg(Color::White));
    f.render_widget(p, area);
}

/// Keybinding help overlay (docs/08 §4: `?` help).
fn render_help(f: &mut Frame, area: Rect) {
    let content = vec![
        "GitX keybindings".to_string(),
        String::new(),
        "  ↑ / k          up".into(),
        "  ↓ / j          down".into(),
        "  ← / h          back".into(),
        "  → / l          open".into(),
        "  Enter          select / drill down".into(),
        "  Esc            close dialog / back".into(),
        "  /              search".into(),
        "  ?              this help".into(),
        "  r              refresh".into(),
        "  q / Ctrl-C     quit".into(),
        String::new(),
        "In a list: j/k scrolls, Enter opens the selected row's detail.".into(),
    ];
    let rect = centered_rect(55, 60, area);
    let list = List::new(content)
        .block(Block::default().title(" Help ").borders(Borders::ALL))
        .highlight_style(Style::default().bg(Color::DarkGray));
    f.render_widget(list, rect);
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
