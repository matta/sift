#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::uninlined_format_args)]
#![deny(unused_crate_dependencies)]
#![deny(unused_extern_crates)]

use std::path::{Path, PathBuf};

use anyhow::Result;
use cli_log::{debug, warn};
use ratatui::backend::CrosstermBackend;
use ratatui::crossterm;
use ratatui::Terminal;
use xilem::view::button;
use xilem::view::flex;
use xilem::EventLoop;
use xilem::WidgetView;
use xilem::Xilem;

mod keys;
mod persist;
mod screen;
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

fn handle_key_event(state: &mut ui_state::State, key_event: crossterm::event::KeyEvent) {
    // TODO: do this combining earlier, properly.
    let key_combination: crokey::KeyCombination = key_event.into();

    if let Some(screen) = state.current_screen.take() {
        state.current_screen =
            Some(screen.handle_key_event(&mut state.common_state, key_combination));
    }
}

fn app_logic(_state: &mut ui_state::State) -> impl WidgetView<ui_state::State> {
    let first_line = flex(
        button("Add task".to_string(), |_state| {
            todo!();
        }),
    );
    first_line
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

    let app = Xilem::new(state, app_logic);
    app.run_windowed(EventLoop::with_user_event(), "First Example".into())
        .unwrap();

    // // Start the main loop.
    // loop {
    //   // Render the user interface.
    //     tui.draw(&mut state)?;
    //     // Handle terminal events.
    //     match tui.next_terminal_event() {
    //         terminal_input::Event::Key(key_event) => {
    //             handle_key_event(&mut state, key_event);
    //         }
    //         terminal_input::Event::Mouse(event) => {
    //             debug!("Mouse({:#?})", event);
    //         }
    //         terminal_input::Event::Tick => {}
    //         terminal_input::Event::Resize(width, height) => {
    //             debug!("Resize({}, {})", width, height);
    //         }
    //     }

    //     match &state.current_screen {
    //         None => {
    //             debug!("no current screen; exiting.");
    //             break;
    //         }
    //         Some(screen) => {
    //             if screen.should_quit(&mut state.common_state) {
    //                 debug!("current screen says quit; exiting.");
    //                 break;
    //             }
    //         }
    //     }
    // }

    // Exit the user interface.
    tui.exit()?;

    // TODO: how to save the state at the end of the program?
    // state.save(save_name)?;

    Ok(())
}
