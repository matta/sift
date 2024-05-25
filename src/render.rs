//! Widget renderer

use ratatui::prelude::Frame;

use crate::{edit_screen, main_screen, ui_state};

pub(crate) fn render(state: &ui_state::State, f: &mut Frame) {
    match &state.current_screen {
        ui_state::Screen::Main => {
            main_screen::render(f, &state.list);
        }
        ui_state::Screen::Edit(edit_state) => {
            edit_screen::render(f, &edit_state);
        }
    }
}
