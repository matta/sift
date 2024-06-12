//! Application updater

use crate::ui_state;

// FIXME: the bool return value here is wonky; make better
#[must_use]
pub(crate) fn handle_key_event(
    state: &mut ui_state::State,
    key_event: crossterm::event::KeyEvent,
) -> bool {
    // TODO: do this combinding earlier, properly.
    let key_combination: crokey::KeyCombination = key_event.into();

    // Hard code a security key to exit the program.  This allows the user to
    // exit the program no matter how badly the key bindings are mishandled.
    if key_event.code == crossterm::event::KeyCode::Esc {
        return false;
    }

    if let Some(screen) = state.current_screen.take() {
        state.current_screen = Some(screen.handle_key_event(key_combination));
    }
    true
}
