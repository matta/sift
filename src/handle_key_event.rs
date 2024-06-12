//! Application updater

use crate::ui_state;

/// The possible actions that can be taken in the application.
///
/// * The `NoChange` variant indicates that no action needs to be taken.
/// * The `AcceptTaskEdit` variant represents accepting an edit to a task, and
///   includes the index of the task and the new task text.
/// * The `SwitchToMainScreen` variant indicates that the application should
///   switch to the main screen.
/// * The `Quit` variant represents the user quitting the application.
// TODO: this is in the wrong module for this enum
#[must_use]
#[derive(PartialEq, Eq)]
pub(crate) enum Action {
    Handled,
    SwitchToMainScreen,
    Quit,
    SwitchToEditScreen(uuid::Uuid, String),
}

pub(crate) fn handle_key_event(
    state: &mut ui_state::State,
    key_event: crossterm::event::KeyEvent,
) -> Action {
    // TODO: do this combinding earlier, properly.
    let key_combination: crokey::KeyCombination = key_event.into();

    // Hard code a security key to exit the program.  This allows the user to
    // exit the program no matter how badly the key bindings are mishandled.
    if key_event.code == crossterm::event::KeyCode::Esc {
        return Action::Quit;
    }

    state.current_screen.handle_key_event(key_combination)
}
