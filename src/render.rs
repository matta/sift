//! Widget renderer

use ratatui::prelude::Frame;

use crate::{edit_screen, main_screen, ui_state};

pub(crate) fn render(state: &ui_state::State, f: &mut Frame) {
    match &state.current_screen {
        ui_state::Screen::Main(main_state) => {
            let mut list_state = main_state.list_state.borrow_mut();
            list_state.select(main_state.common_state.list.selected());
            main_screen::render(
                f,
                &main_state.common_state.list,
                &mut list_state,
            );
        }
        ui_state::Screen::Edit(edit_state) => {
            edit_screen::render(f, edit_state);
        }
    }
}
