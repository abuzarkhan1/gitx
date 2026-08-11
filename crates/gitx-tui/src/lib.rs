pub mod app;
pub mod events;
pub mod index_backed;
pub mod terminal;
pub mod ui;
pub mod views;

use crate::app::{App, View};
use crate::events::{Event, EventHandler};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
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
                // Background repository data may have landed (docs/08 loading
                // progress): the UI stays responsive while it computes.
                Event::Tick => {
                    app.poll_load();
                }
                Event::Resize(_, _) => {}
            }
        }
    }

    terminal::restore_terminal()?;
    Ok(())
}

fn handle_key(app: &mut App, search_mode: &mut bool, key: KeyEvent) {
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
            if app.show_help {
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
                    app.scroll_down();
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
                    app.scroll_up();
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
        KeyCode::Char(c) if key.modifiers.contains(KeyModifiers::CONTROL) && c == 'c' => {
            app.quit();
        }
        _ => {}
    }
}
