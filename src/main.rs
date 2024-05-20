/*!
This is a toy todo list application I have written to explore Rust.
*/
#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::uninlined_format_args)]

pub mod event;
pub mod persist;
pub mod ui_state;
pub mod tui;
pub mod ui;
pub mod update;

use anyhow::Result;
use event::Event;
use ratatui::{backend::CrosstermBackend, Terminal};
use tui::Tui;
use update::update;

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
    let mut disposition = update::Disposition::Continue;
    while disposition == update::Disposition::Continue {
        // Render the user interface.
        tui.draw(&mut state)?;
        // Handle events.
        disposition = match tui.event_reader.next()? {
            Event::Key(key_event) => update(&mut state, key_event),
            Event::Tick | Event::Mouse(_) | Event::Resize(_, _) => update::Disposition::Continue,
        };
    }

    // Exit the user interface.
    tui.exit()?;

    state.save(save_name)?;

    Ok(())
}
