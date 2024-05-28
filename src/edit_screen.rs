use std::borrow::Cow;

use ratatui::Frame;
use tui_prompts::{State, TextPrompt};

use crate::handle_key_event::Action;
use crate::ui_state;
use crate::ui_state::EditScreenState;

pub fn render(f: &mut Frame, edit_state: &EditScreenState) {
    let prompt = TextPrompt::new(Cow::Borrowed("edit"));
    f.render_stateful_widget(prompt, f.size(), &mut edit_state.text_state.borrow_mut());
    let (x, y) = edit_state.text_state.borrow().cursor();
    f.set_cursor(x, y);
}

pub fn handle_key_event(
    edit_state: &mut ui_state::EditScreenState,
    key_event: crossterm::event::KeyEvent,
) -> Action {
    let mut text_state = edit_state.text_state.borrow_mut();
    assert!(text_state.is_focused());
    text_state.handle_key_event(key_event);
    match text_state.status() {
        tui_prompts::Status::Pending => Action::Handled,
        tui_prompts::Status::Aborted => {
            // TODO: When aborting a new item, delete it.
            Action::SwitchToMainScreen
        }
        tui_prompts::Status::Done => {
            Action::AcceptTaskEdit(edit_state.index, text_state.value().into())
        }
    }
}
