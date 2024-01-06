use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::App;

pub fn update(app: &mut App, key_event: KeyEvent) {
    match key_event.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.should_quit = true;
        }
        KeyCode::Char('c') | KeyCode::Char('C') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                app.should_quit = true;
            }
        }

        KeyCode::Right | KeyCode::Char('j') => app.counter += 1,
        KeyCode::Left | KeyCode::Char('k') => app.counter -= 1,
        _ => {}
    };
}
