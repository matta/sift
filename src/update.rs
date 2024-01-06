use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tui_textarea::TextArea;

use crate::app::{App, EditState, Screen};

pub(crate) fn update(app: &mut App, key_event: KeyEvent) {
    match &mut app.state.screen {
        Screen::Main => match key_event.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                app.should_quit = true;
            }
            KeyCode::Char('c') | KeyCode::Char('C') => {
                if key_event.modifiers == KeyModifiers::CONTROL {
                    app.should_quit = true;
                }
            }

            KeyCode::Char(' ') => app.state.list.toggle(),
            KeyCode::Char('e') => {
                if let Some(index) = app.state.list.state.selected() {
                    let textarea: TextArea<'_> = app.state.list.items[index].title.lines().into();
                    let edit_state = EditState { index, textarea };
                    app.state.screen = Screen::Edit(edit_state);
                }
            }

            KeyCode::Left | KeyCode::Char('h') => app.state.list.unselect(),
            KeyCode::Down | KeyCode::Char('j') => app.state.list.next(),
            KeyCode::Up | KeyCode::Char('k') => app.state.list.previous(),

            _ => {}
        },
        Screen::Edit(edit_state) => match key_event.code {
            KeyCode::Esc => {
                app.state.screen = Screen::Main;
            }
            KeyCode::Char('s') if key_event.modifiers == KeyModifiers::CONTROL => {
                app.state.list.items[edit_state.index].title = edit_state
                    .textarea
                    .lines()
                    .iter()
                    .cloned()
                    .collect::<String>();
                app.state.screen = Screen::Main;
            }
            _ => {
                edit_state.textarea.input(key_event);
            }
        },
    }
}
