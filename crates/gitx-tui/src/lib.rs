pub mod app;
pub mod events;
pub mod index_backed;
pub mod terminal;
pub mod ui;
pub mod views;

use crate::app::{App, View};
use crate::events::{Event, EventHandler};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEventKind};
use std::time::Duration;

pub async fn run() -> Result<()> {
    let mut terminal = terminal::setup_terminal()?;
    let mut app = App::new();
    let mut events = EventHandler::new(Duration::from_millis(250));
    // When true, keystrokes edit the search query instead of navigating.
    let mut search_mode = false;

    while app.running {
        terminal.draw(|f| ui::render(f, &mut app))?;

        if let Some(event) = events.next().await {
            match event {
                Event::Key(key_event) => handle_key(&mut app, &mut search_mode, key_event),
                // Background repository data or FTS search results may have
                // landed (docs/08 loading progress / #20 async search); the
                // loading spinner advances every tick.
                Event::Tick => {
                    app.poll_load();
                    app.poll_search();
                    // Advance the spinner while the background loader OR an FTS
                    // search is running (docs/08 loading progress / #20).
                    if app.loading || app.search_pending {
                        app.load_frame = app.load_frame.wrapping_add(1);
                    }
                }
                Event::Mouse(mouse) => handle_mouse(&mut app, &search_mode, mouse),
                Event::Resize(_, _) => {}
            }
        }
    }

    terminal::restore_terminal()?;
    Ok(())
}

/// Mouse support (docs/08 #17): scroll wheel scrolls the current list, a left
/// click on the sidebar jumps to that view.
fn handle_mouse(app: &mut App, search_mode: &bool, mouse: crossterm::event::MouseEvent) {
    if *search_mode {
        return;
    }
    match mouse.kind {
        MouseEventKind::ScrollUp => {
            if app.in_content {
                if app.current_view == View::Detail {
                    app.detail_scroll = app.detail_scroll.saturating_sub(1);
                } else {
                    app.cursor_up();
                }
            }
        }
        MouseEventKind::ScrollDown => {
            if app.in_content {
                if app.current_view == View::Detail {
                    app.detail_scroll = app.detail_scroll.saturating_add(1);
                } else {
                    app.cursor_down();
                }
            }
        }
        // Sidebar occupies the first `sidebar_width` columns (docs/08 §5
        // responsive layout: the width adapts to the terminal). Row 1 = header
        // offset, each nav item is one row inside the bordered list.
        MouseEventKind::Down(MouseButton::Left)
            if mouse.column < crate::ui::sidebar_width(app.width)
                && mouse.row >= 3
                && mouse.row <= 16 =>
        {
            app.nav_index = (mouse.row - 3) as usize;
            app.select_nav();
        }
        _ => {}
    }
}

fn handle_key(app: &mut App, search_mode: &mut bool, key: KeyEvent) {
    // Ctrl+C always quits — even while typing a search query. Raw mode
    // delivers it as a key event, and it must beat the view-jump arm below
    // (otherwise Ctrl+C while navigating would open the Commits view).
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        app.quit();
        return;
    }
    if *search_mode {
        match key.code {
            KeyCode::Char(c) => {
                app.search_query.push(c);
                app.run_search();
            }
            KeyCode::Backspace => {
                app.search_query.pop();
                app.run_search();
            }
            KeyCode::Esc | KeyCode::Enter => {
                *search_mode = false;
                if app.current_view != View::Search {
                    app.current_view = View::Search;
                }
            }
            _ => {}
        }
        return;
    }

    match key.code {
        KeyCode::Char('q') => app.quit(),
        KeyCode::Esc => {
            // Esc cancels an in-flight reload (docs/08 §6 cancellation hint).
            if app.loading {
                app.cancel_loading();
            } else if app.show_help {
                app.show_help = false;
            } else if app.error.is_some() {
                app.error = None;
            } else if app.current_view == View::Detail {
                // Close the detail back to its originating panel.
                app.close_detail();
            } else if app.in_content {
                // Leave the content view back to sidebar navigation.
                app.in_content = false;
                app.scroll = 0;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.in_content {
                if app.current_view == View::Detail {
                    app.detail_scroll = app.detail_scroll.saturating_add(1);
                } else {
                    // j/k moves the selection (docs/08 #20: Enter opens the
                    // selected row); the window scrolls only when needed.
                    app.cursor_down();
                }
            } else {
                app.next_nav();
            }
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if app.in_content {
                if app.current_view == View::Detail {
                    app.detail_scroll = app.detail_scroll.saturating_sub(1);
                } else {
                    app.cursor_up();
                }
            } else {
                app.prev_nav();
            }
        }
        KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => {
            if app.current_view == View::Detail {
                // In detail, Enter is a no-op (Esc/← closes).
            } else if app.in_content {
                // Drill down: open the selected row's detail.
                app.open_detail();
            } else {
                app.select_nav();
            }
        }
        KeyCode::Left | KeyCode::Char('h') => {
            if app.current_view == View::Detail {
                app.close_detail();
            } else if app.in_content {
                app.in_content = false;
                app.scroll = 0;
            }
        }
        KeyCode::Char('r') => app.reload(),
        KeyCode::Char('/') => {
            *search_mode = true;
            app.current_view = View::Search;
            app.sync_nav();
            app.search_query.clear();
            app.search_results = None;
        }
        KeyCode::Char('?') => app.show_help = !app.show_help,
        // Hotspots sort toggle (docs/08 sortable table): s changes the sort
        // while the Hotspots view is open.
        KeyCode::Char('s') if app.current_view == View::Hotspots && app.in_content => {
            app.cycle_hotspot_sort();
            app.selected = 0;
            app.scroll = 0;
        }
        // Number-key view jumps (docs/08 #18): 1–9 map to the first nine views.
        KeyCode::Char('1'..='9') if !app.in_content => {
            let n = match key.code {
                KeyCode::Char(c) => c.to_digit(10).unwrap_or(1) as usize - 1,
                _ => 0,
            };
            jump_index(app, n);
        }
        // Direct view-jump keys (docs/08 #18): work from anywhere.
        KeyCode::Char(c) if !app.in_content && app.current_view != View::Detail => {
            jump_view(app, c);
        }
        _ => {}
    }
}

/// View-jump shortcuts (docs/08 #18): o Overview, t Timeline, c Commits,
/// b Branches, f Files, u Contributors, s Hotspots, w Ownership, a
/// Architecture, d Dependencies, x Risk, e Health, v Recovery.
fn jump_view(app: &mut App, c: char) {
    let view = match c {
        'o' => View::Overview,
        't' => View::Timeline,
        'c' => View::Commits,
        'b' => View::Branches,
        'f' => View::Files,
        'u' => View::Contributors,
        's' => View::Hotspots,
        'w' => View::Ownership,
        'a' => View::Architecture,
        'd' => View::Dependencies,
        'x' => View::Risk,
        'e' => View::Health,
        'v' => View::Recovery,
        _ => return,
    };
    app.current_view = view;
    app.sync_nav();
    app.in_content = true;
    app.scroll = 0;
    app.selected = 0;
    app.nav_used = true;
}

/// Number-key view jumps (docs/08 #18): 1–9 map to the first nine views.
fn jump_index(app: &mut App, index: usize) {
    let view = match index {
        0 => View::Overview,
        1 => View::Timeline,
        2 => View::Commits,
        3 => View::Branches,
        4 => View::Files,
        5 => View::Contributors,
        6 => View::Hotspots,
        7 => View::Ownership,
        8 => View::Architecture,
        _ => return,
    };
    app.current_view = view;
    app.sync_nav();
    app.in_content = true;
    app.scroll = 0;
    app.selected = 0;
    app.nav_used = true;
}
