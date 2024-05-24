/*!
This is a toy todo list application I have written to explore Rust.
*/
#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::uninlined_format_args)]
#![deny(unused_crate_dependencies)]
#![deny(unused_extern_crates)]

use anyhow::Result;
use ratatui::{backend::CrosstermBackend, Terminal};

use event::Event;
use tui::Tui;
use update::update;

pub mod event;
pub mod persist;
pub mod tui;
pub mod ui;
pub mod ui_state;
pub mod update;

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
    let events = event::Reader::new(250);
    let mut tui = Tui::new(terminal, events);
    tui.enter()?;

    // Start the main loop.
    loop {
        // Render the user interface.
        tui.draw(&state)?;
        // Handle events.
        let disposition = match tui.event_reader.next()? {
            Event::Key(key_event) => update(&mut state, key_event),
            Event::Tick | Event::Mouse(_) | Event::Resize(_, _) => update::Action::NoChange,
        };
        match disposition {
            update::Action::AcceptTaskEdit(index, new_title) => {
                state.list.tasks.tasks[index].title = new_title;
                state.current_screen = ui_state::Screen::Main;
            }
            update::Action::SwitchToMainScreen => {
                state.current_screen = ui_state::Screen::Main;
            }
            update::Action::Quit => {
                break;
            }
            update::Action::NoChange => {}
        }
    }

    // Exit the user interface.
    tui.exit()?;

    state.save(save_name)?;

    Ok(())
}
