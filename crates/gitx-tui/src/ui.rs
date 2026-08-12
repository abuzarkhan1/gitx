use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

use crate::app::{App, View};
use crate::views;
use crate::views::theme;

/// Minimum usable terminal size (docs/08 §6 responsive small-terminal layout).
/// Below this the layout collapses; show guidance instead of broken panels.
pub const MIN_WIDTH: u16 = 60;
pub const MIN_HEIGHT: u16 = 20;

/// Navigation sidebar width for the current terminal width (docs/08 §5
/// responsive layout): wide terminals get the full nav, narrow ones collapse
/// it so the content keeps the space. Shared with the mouse handler so a
/// sidebar click lines up with what is drawn.
pub fn sidebar_width(total_width: u16) -> u16 {
    if total_width >= 110 {
        20
    } else if total_width >= 80 {
        16
    } else {
        12
    }
}

pub fn render(f: &mut Frame, app: &mut App) {
    let area = f.area();
    if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
        render_too_small(f, area);
        return;
    }
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header (with breadcrumb)
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

/// Header with branding + a "you are here" breadcrumb (docs/08 #19): the
/// current view name is always visible in the content area's top-left corner.
fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let t = theme::global();
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
    let state_color = if state == "clean" {
        Color::Green
    } else {
        Color::Yellow
    };
    let view_name = view_label(app.current_view);

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            " GitX ",
            Style::default().fg(t.accent).add_modifier(Modifier::BOLD),
        ),
        Span::raw(format!(" {name}   ")),
        Span::styled(branch.to_string(), Style::default().fg(Color::Cyan)),
        Span::raw("  "),
        Span::styled(state, Style::default().fg(state_color)),
        Span::raw("  "),
        Span::styled(
            format!("▸ {view_name}"),
            Style::default().fg(t.accent).add_modifier(Modifier::BOLD),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL))
    .style(Style::default().fg(t.fg));
    f.render_widget(header, area);
}

fn view_label(view: View) -> &'static str {
    match view {
        View::Overview => "Overview",
        View::Timeline => "Timeline",
        View::Commits => "Commits",
        View::Branches => "Branches",
        View::Files => "Files",
        View::Contributors => "Contributors",
        View::Hotspots => "Hotspots",
        View::Ownership => "Ownership",
        View::Architecture => "Architecture",
        View::Dependencies => "Dependencies",
        View::Risk => "Risk",
        View::Health => "Health",
        View::Recovery => "Recovery",
        View::Search => "Search",
        View::Graph => "Graph",
        View::Detail => "Detail",
    }
}

fn render_main(f: &mut Frame, app: &mut App, area: Rect) {
    // Collapse the navigation sidebar on narrow terminals (docs/08 §5): the
    // content area keeps the space, and view-jump keys still reach every view.
    let width = sidebar_width(area.width);
    app.width = area.width;
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(width), Constraint::Min(0)])
        .split(area);

    render_navigation(f, app, chunks[0]);
    render_content(f, app, chunks[1]);
}

fn render_navigation(f: &mut Frame, app: &mut App, area: Rect) {
    let t = theme::global();
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
        "Graph",
    ];

    // Truncate long labels on narrow terminals so nothing overflows the
    // collapsed sidebar (docs/08 §5: never silently clip important info —
    // the full name is one jump-key away).
    let max_label = area.width.saturating_sub(5) as usize;
    let items: Vec<ListItem> = views
        .into_iter()
        .map(|v| {
            let label: String = if v.chars().count() > max_label {
                let mut s: String = v.chars().take(max_label.saturating_sub(1)).collect();
                s.push('…');
                s
            } else {
                v.to_string()
            };
            ListItem::new(label)
        })
        .collect();

    let mut state = ListState::default();
    state.select(Some(app.nav_index));

    let list = List::new(items)
        .block(Block::default().title(" Navigation ").borders(Borders::ALL))
        .highlight_style(
            Style::default()
                .bg(t.sel_bg)
                .fg(t.fg)
                .add_modifier(Modifier::BOLD),
        )
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
            app.commit_files.as_deref(),
            app.scroll,
            app.selected,
        ),
        View::Branches => views::branches::render(
            f,
            area,
            app.branches.as_deref(),
            app.branch_tips.as_deref(),
            app.branch_intel.as_deref(),
            app.loading,
            app.scroll,
            app.selected,
        ),
        View::Files => views::files::render(
            f,
            area,
            app.hotspots.as_ref(),
            app.loading,
            app.scroll,
            app.selected,
        ),
        View::Contributors => views::contributors::render(
            f,
            area,
            app.contributors.as_deref(),
            app.loading,
            app.scroll,
            app.selected,
        ),
        View::Hotspots => views::hotspots::render(
            f,
            area,
            app.hotspots.as_ref(),
            app.hotspot_sort,
            app.loading,
            app.scroll,
            app.selected,
        ),
        View::Ownership => views::ownership::render(
            f,
            area,
            app.hotspots.as_ref(),
            app.loading,
            app.scroll,
            app.selected,
        ),
        View::Architecture => views::architecture::render(
            f,
            area,
            app.arch_diff.as_ref(),
            app.hotspots.as_ref(),
            app.loading,
            app.scroll,
            app.selected,
        ),
        View::Dependencies => views::dependencies::render(
            f,
            area,
            app.dependencies.as_deref(),
            app.loading,
            app.scroll,
            app.selected,
        ),
        View::Risk => views::risk::render(
            f,
            area,
            app.hotspots.as_ref(),
            app.loading,
            app.scroll,
            app.selected,
        ),
        View::Health => views::health::render(f, area, app),
        View::Recovery => views::recovery::render(
            f,
            area,
            app.recovery.as_ref(),
            app.loading,
            app.scroll,
            app.selected,
        ),
        View::Search => views::search::render(
            f,
            area,
            &app.search_query,
            app.search_pending,
            app.search_results.as_deref(),
            app.in_content,
            app.scroll,
            app.selected,
        ),
        View::Graph => views::graph::render(
            f,
            area,
            app.graph_summary.as_deref(),
            app.loading,
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
    app.last_row_count = row_count;
    // Visible rows inside the content list (matches common::render_scrollable
    // and views::search::render): keeps the cursor highlight in the window.
    app.visible = area.height.saturating_sub(2).max(1) as usize;
    if app.selected >= row_count && row_count > 0 {
        app.selected = row_count - 1;
    }

    if let Some(ref err) = app.error {
        // Structured error overlay (docs/08 §8): the message plus a
        // reason and an actionable suggestion instead of a bare string.
        // 42% height fits the full Reason/Suggested-action block even on
        // short terminals (the block wraps to ~10 rows).
        let err_rect = centered_rect(72, 42, area);
        let lines: Vec<Line> = error_overlay_lines(err);
        let err_p = Paragraph::new(lines)
            .block(Block::default().title(" Error ").borders(Borders::ALL))
            .style(Style::default().fg(Color::Red));
        f.render_widget(err_p, err_rect);
    }
}

/// Contextual status bar (docs/08 #20 per-row hints + loading progress):
/// shows an animated spinner while data/search loads, the visible-row range,
/// and what the selected row will do when Enter is pressed.
fn render_status_bar(f: &mut Frame, app: &mut App, area: Rect) {
    let t = theme::global();
    let spinner = spinner_frame(app.load_frame);
    let status = if app.loading {
        // docs/08 §6: operation name + processed/total + cancellation hint.
        if app.current_view == View::Search {
            format!(
                " {spinner} {} ({}/{}) — search works while data loads  |  Esc cancel  q quit ",
                app.load_phase, app.load_step, app.load_total
            )
        } else {
            format!(
                " {spinner} {} ({}/{})  |  Esc cancel  q quit ",
                app.load_phase, app.load_step, app.load_total
            )
        }
    } else if app.show_help {
        " Esc Close help  q Quit ".to_string()
    } else if app.current_view == View::Detail {
        format!(
            " j/k Scroll  {}   Esc/← Close detail  q Quit ",
            visible_range(app, f.area().height)
        )
    } else if app.current_view == View::Search {
        if app.search_pending {
            format!(
                " {spinner} Searching index + code... (commits, files, authors, branches, tags, symbols) "
            )
        } else if app.search_query.trim().is_empty() {
            " Type to search  Enter/← Done  q Quit ".to_string()
        } else {
            " ↑↓ Move  Enter Open result  Esc Done  q Quit ".to_string()
        }
    } else if app.current_view == View::Hotspots {
        format!(
            " s Sort: {}   {}   Enter View file   / Search   r Refresh   q Quit ",
            match app.hotspot_sort {
                0 => "score",
                1 => "changes",
                _ => "churn",
            },
            visible_range(app, f.area().height)
        )
    } else if app.in_content {
        format!(
            " j/k Scroll  {}   Enter Open  Esc/← Back  / Search  r Refresh  q Quit ",
            visible_range(app, f.area().height)
        )
    } else if !app.nav_used {
        " ↑↓ Navigate · Enter Open a view · / Search · ? Help · r Refresh · q Quit ".to_string()
    } else {
        " ↑↓ Navigate  Enter Open  / Search  ? Help  r Refresh  q Quit ".to_string()
    };

    let p = Paragraph::new(status).style(Style::default().bg(t.status_bg).fg(Color::White));
    f.render_widget(p, area);
}

/// Structured error overlay (docs/08 §8): the raw message under “Reason:”
/// plus a heuristic “Suggested action:” so errors are actionable. The
/// heuristic covers the failure modes the CLI/TUI actually hit; anything
/// unrecognized falls back to the generic refresh-and-retry guidance.
fn error_overlay_lines(err: &str) -> Vec<Line<'static>> {
    let reason = err.lines().next().unwrap_or(err).to_string();
    let suggested = if reason.contains("not inside a Git repository") {
        "Run GitX from inside a Git repository, or initialize one with `git init`."
    } else if reason.contains("index") && reason.contains("schema") {
        "The index was built by a newer GitX. Upgrade GitX, or rebuild it with `gitx refresh`."
    } else if reason.contains("permission") || reason.contains("denied") {
        "Check that GitX can read the repository files (file permissions)."
    } else if reason.contains("lock") {
        "Another Git process may hold a lock. Close it, then run `gitx refresh`."
    } else {
        "Run `gitx refresh` to rebuild the index, or check `gitx --help`."
    };
    vec![
        Line::from(Span::styled(
            "Unable to complete the operation.",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )),
        Line::default(),
        Line::from(Span::styled(
            "Reason:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!("  {reason}"),
            Style::default().fg(Color::White),
        )),
        Line::default(),
        Line::from(Span::styled(
            "Suggested action:",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!("  {suggested}"),
            Style::default().fg(Color::White),
        )),
        Line::default(),
        Line::from(Span::styled(
            "  (press Esc to dismiss)",
            Style::default().fg(Color::DarkGray),
        )),
    ]
}

/// Spinner frames (docs/08 loading progress / #20 async search): animated
/// while the background loader or an FTS search is running.
fn spinner_frame(frame: u8) -> &'static str {
    const FRAMES: [&str; 8] = ["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"];
    FRAMES[(frame as usize) % FRAMES.len()]
}

/// “showing a–b of N” scroll-position indicator (docs/08): the visible row
/// range derived from the scroll offset, the content height, and the view's
/// total row count.
fn visible_range(app: &App, total_height: u16) -> String {
    let row_count = app.last_row_count;
    if row_count == 0 {
        return "showing 0 rows".to_string();
    }
    // Header (3 rows incl. borders) + status bar (1) + list borders (2).
    let content_h = total_height.saturating_sub(4);
    let visible = (content_h.saturating_sub(2)).max(1) as usize;
    let scroll = if app.current_view == View::Detail {
        app.detail_scroll
    } else {
        app.scroll
    };
    let first = scroll + 1;
    let last = (scroll + visible).min(row_count);
    format!("showing {first}–{last} of {row_count}")
}

/// Keybinding help overlay (docs/08 §4: `?` help).
fn render_help(f: &mut Frame, area: Rect) {
    let t = theme::global();
    let content = vec![
        Line::from(Span::styled(
            "GitX keybindings",
            Style::default().fg(t.accent).add_modifier(Modifier::BOLD),
        )),
        Line::default(),
        Line::raw("  ↑ / k          up"),
        Line::raw("  ↓ / j          down"),
        Line::raw("  ← / h          back"),
        Line::raw("  → / l          open"),
        Line::raw("  Enter          select / drill down"),
        Line::raw("  Esc            close dialog / back"),
        Line::raw("  /              search"),
        Line::raw("  ?              this help"),
        Line::raw("  r              refresh"),
        Line::raw("  s              (Hotspots) change sort"),
        Line::raw("  1–9, o/t/c/b/f/u/s/w/a/d/x/e/v   jump to a view"),
        Line::raw("  q / Ctrl-C     quit"),
        Line::default(),
        Line::raw("In a list: j/k scrolls, Enter opens the selected row's detail."),
    ];
    let rect = centered_rect(55, 60, area);
    let list = List::new(content)
        .block(Block::default().title(" Help ").borders(Borders::ALL))
        .highlight_style(Style::default().bg(t.sel_bg));
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
