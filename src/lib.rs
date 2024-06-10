#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::uninlined_format_args)]
#![deny(unused_crate_dependencies)]
#![deny(unused_extern_crates)]

use anyhow::Result;
use cli_log::{debug, warn};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::path::{Path, PathBuf};

mod edit_screen;
mod handle_key_event;
mod keys;
mod main_screen;
mod persist;
mod render;
mod terminal_input;
mod tui;
mod ui_state;

#[must_use]
pub fn save_name() -> PathBuf {
    let mut path = if let Some(home) = dirs::home_dir() {
        home
    } else {
        PathBuf::new()
    };
    path.push(".sift.sift");
    path
}

/// # Errors
///
/// TODO: write me
pub fn run(save_name: &Path) -> Result<()> {
    // Create an application.
    let mut state = match ui_state::State::load(save_name) {
        Ok(app) => {
            debug!("loaded state from disk");
            app
        }
        Err(error) => {
            warn!(
                "loading todos failed: {}; using a default set of todos",
                error
            );
            ui_state::State::new()
        }
    };

    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(std::io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = terminal_input::Reader::new(250);
    let mut tui = tui::Tui::new(terminal, events);
    tui.enter()?;

    // Start the main loop.
    loop {
        // Render the user interface.
        tui.draw(&state)?;
        // Handle terminal events.
        let disposition = match tui.next_terminal_event() {
            terminal_input::Event::Key(key_event) => {
                handle_key_event::handle_key_event(&mut state, key_event)
            }
            terminal_input::Event::Tick
            | terminal_input::Event::Mouse(_)
            | terminal_input::Event::Resize(_, _) => {
                handle_key_event::Action::Handled
            }
        };
        match disposition {
            handle_key_event::Action::AcceptTaskEdit(index, new_title) => {
                let mut common_state = state.current_screen.take_common_state();
                common_state.list.tasks.tasks[index].title = new_title;
                state.current_screen =
                    ui_state::Screen::Main(ui_state::MainScreenState {
                        common_state,
                    });
            }
            handle_key_event::Action::SwitchToMainScreen => {
                state.current_screen =
                    ui_state::Screen::Main(ui_state::MainScreenState {
                        common_state: state.current_screen.take_common_state(),
                    });
            }
            handle_key_event::Action::SwitchToEditScreen(index, title) => {
                let text_state = tui_prompts::TextState::new()
                    .with_value(std::borrow::Cow::Owned(title))
                    .with_focus(tui_prompts::FocusState::Focused);
                let common_state = state.current_screen.take_common_state();
                let edit_state = ui_state::EditScreenState {
                    common_state,
                    index,
                    text_state: std::cell::RefCell::new(text_state),
                };
                state.current_screen = ui_state::Screen::Edit(edit_state);
            }
            handle_key_event::Action::Quit => {
                break;
            }
            handle_key_event::Action::Handled => {}
        }
    }

    // Exit the user interface.
    tui.exit()?;

    state.save(save_name)?;
    Ok(())
}
