use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::widgets::{Block, Borders};
use tui_textarea::TextArea;

use crate::app::{App, EditState, Screen, Todo};

pub(crate) fn update(app: &mut App, key_event: KeyEvent) {
    match &mut app.state.screen {
        Screen::Main => match key_event.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                app.should_quit = true;
            }
            KeyCode::Char('c') | KeyCode::Char('C')
                if key_event.modifiers == KeyModifiers::CONTROL =>
            {
                app.should_quit = true;
            }
            KeyCode::Char(' ') => {
                app.state.list.toggle();
            }
            KeyCode::Char('e') => {
                edit(app);
            }
            KeyCode::Left | KeyCode::Char('h') => {
                app.state.list.unselect();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                app.state.list.next();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                app.state.list.previous();
            }
            KeyCode::Char('a') => {
                add(app);
            }
            KeyCode::Char('D') => {
                delete(app);
            }
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

fn delete(app: &mut App) {
    let list = &mut app.state.list;
    if let Some(index) = list.state.selected() {
        // Decrement the selected items by the number of todo
        // items that will be deleted.
        let count = list
            .items
            .iter()
            .enumerate()
            .filter(|(i, todo)| *i < index && todo.done)
            .count();
        *list.state.selected_mut() = Some(index - count);
    }
    list.items.retain(|todo| !todo.done);
}

fn add(app: &mut App) {
    let list = &mut app.state.list;
    let index = list.state.selected().unwrap_or(0);
    *list.state.selected_mut() = Some(index);
    list.items.insert(
        index,
        Todo {
            title: String::new(),
            done: false,
        },
    );
    edit(app);
}

fn edit(app: &mut App) {
    let list = &mut app.state.list;
    if let Some(index) = list.state.selected() {
        let mut textarea: TextArea<'static> = list.items[index].title.lines().into();
        textarea.set_block(Block::default().borders(Borders::ALL).title("Edit"));
        let edit_state = EditState { index, textarea };
        app.state.screen = Screen::Edit(edit_state);
    }
}
