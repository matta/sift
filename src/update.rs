use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::App;

pub(crate) fn update(app: &mut App, key_event: KeyEvent) {
    match key_event.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.should_quit = true;
        }
        KeyCode::Char('c') | KeyCode::Char('C') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                app.should_quit = true;
            }
        }

        KeyCode::Char(' ') => app.list.toggle(),

        KeyCode::Left | KeyCode::Char('h') => app.list.unselect(),
        KeyCode::Down | KeyCode::Char('j') => app.list.next(),
        KeyCode::Up | KeyCode::Char('k') => app.list.previous(),

        _ => {}
    };
}
