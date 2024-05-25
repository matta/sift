/*!
This is a toy todo list application I have written to explore Rust.
*/
#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::uninlined_format_args)]
#![deny(unused_crate_dependencies)]
#![deny(unused_extern_crates)]

use anyhow::Result;
use ratatui::{backend::CrosstermBackend, Terminal};

use tui::Tui;

pub mod handle_key_event;
pub mod persist;
pub mod terminal_input;
pub mod tui;
pub mod ui;
pub mod ui_state;

fn main() -> Result<()> {
    let save_name = "sift.sift";
    // Create an application.
    let mut state = if let Ok(app) = ui_state::State::load(save_name) {
        app
    } else {
        ui_state::State::new()
    };

    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(std::io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = terminal_input::Reader::new(250);
    let mut tui = Tui::new(terminal, events);
    tui.enter()?;

    // Start the main loop.
    loop {
        // Render the user interface.
        tui.draw(&state)?;
        // Handle terminal events.
        let disposition = match tui.next_terminal_event()? {
            terminal_input::Event::Key(key_event) => {
                handle_key_event::handle_key_event(&mut state, key_event)
            }
            terminal_input::Event::Tick
            | terminal_input::Event::Mouse(_)
            | terminal_input::Event::Resize(_, _) => handle_key_event::Action::NoChange,
        };
        match disposition {
            handle_key_event::Action::AcceptTaskEdit(index, new_title) => {
                state.list.tasks.tasks[index].title = new_title;
                state.current_screen = ui_state::Screen::Main;
            }
            handle_key_event::Action::SwitchToMainScreen => {
                state.current_screen = ui_state::Screen::Main;
            }
            handle_key_event::Action::Quit => {
                break;
            }
            handle_key_event::Action::NoChange => {}
        }
    }

    // Exit the user interface.
    tui.exit()?;

    state.save(save_name)?;

    Ok(())
}
