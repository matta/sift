//! Terminal user interface

use std::{io, panic};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};

use crate::terminal_input;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Error writing to terminal")]
    TerminalWrite(#[source] std::io::Error),
}

pub type CrosstermTerminal =
    ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stderr>>;

/// Representation of a terminal user interface.
///
/// It is responsible for setting up the terminal, initializing the interface
/// and handling the draw events.
pub(crate) struct Tui {
    /// Interface to the Terminal.
    terminal: CrosstermTerminal,
    /// Terminal event reader.
    event_reader: terminal_input::Reader,
}

impl Tui {
    /// Constructs a new instance of [`Tui`].
    pub fn new(
        terminal: CrosstermTerminal,
        event_reader: terminal_input::Reader,
    ) -> Self {
        Self {
            terminal,
            event_reader,
        }
    }

    /// Initializes the terminal interface.
    ///
    /// It enables the raw mode and sets terminal properties.
    pub fn enter(&mut self) -> Result<(), Error> {
        let mut func = || {
            terminal::enable_raw_mode()?;
            crossterm::execute!(
                io::stderr(),
                EnterAlternateScreen,
                EnableMouseCapture
            )?;

            // Define a custom panic hook to reset the terminal properties.
            // This way, you won't have your terminal messed up if an unexpected error happens.
            let panic_hook = panic::take_hook();
            panic::set_hook(Box::new(move |panic| {
                Self::reset().expect("failed to reset the terminal");
                panic_hook(panic);
            }));

            self.terminal.hide_cursor()?;
            self.terminal.clear()?;
            Ok(())
        };
        func().map_err(Error::TerminalWrite)
    }

    /// Draw the terminal interface by rendering the widgets.
    pub fn draw(
        &mut self,
        state: &mut crate::ui_state::State,
    ) -> Result<(), Error> {
        self.terminal
            .draw(|frame| {
                if let Some(screen) = state.current_screen.take() {
                    state.current_screen = Some(screen.render(frame));
                }
            })
            .map_err(Error::TerminalWrite)?;
        Ok(())
    }

    /// Resets the terminal interface.
    ///
    /// This function is also used for the panic hook to revert
    /// the terminal properties if unexpected errors occur.
    fn reset() -> Result<(), Error> {
        terminal::disable_raw_mode().map_err(Error::TerminalWrite)?;
        crossterm::execute!(
            io::stderr(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )
        .map_err(Error::TerminalWrite)?;
        Ok(())
    }

    /// Exits the terminal interface.
    ///
    /// It disables the raw mode and reverts back the terminal properties.
    pub fn exit(&mut self) -> Result<(), Error> {
        Self::reset()?;
        self.terminal.show_cursor().map_err(Error::TerminalWrite)?;
        Ok(())
    }

    pub fn next_terminal_event(&mut self) -> terminal_input::Event {
        self.event_reader.next()
    }
}
