use std::borrow::Cow;

use chrono::Datelike;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tui_prompts::FocusState;
use tui_prompts::State as _;
use tui_prompts::TextState;
use uuid::Uuid;

use crate::app::{App, EditState, Screen};
use crate::persist::Task;

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
            KeyCode::Char('s') => {
                snooze(app);
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
        Screen::Edit(edit_state) => {
            let text_state = &mut edit_state.text_state;
            assert!(text_state.is_focused());
            text_state.handle_key_event(key_event);
            match text_state.status() {
                tui_prompts::Status::Pending => {}
                tui_prompts::Status::Aborted => {
                    // TODO: When aborting a new item, delete it.
                    app.state.screen = Screen::Main;
                }
                tui_prompts::Status::Done => {
                    let task = app
                        .state
                        .list
                        .tasks
                        .tasks
                        .iter_mut()
                        .nth(edit_state.index)
                        .unwrap()
                        .1;
                    task.title = text_state.value().into();
                    app.state.screen = Screen::Main;
                }
            }
        }
    }
}

fn delete(app: &mut App) {
    let list = &mut app.state.list;
    if let Some(index) = list.state.selected() {
        // Decrement the selected items by the number of todo
        // items that will be deleted.
        let count = list
            .tasks
            .tasks
            .iter()
            .enumerate()
            .filter(|(i, (_, todo))| *i < index && todo.completed)
            .count();
        *list.state.selected_mut() = Some(index - count);
    }
    list.tasks.tasks.retain(|_, todo| !todo.completed);
}

fn snooze(app: &mut App) {
    let list = &mut app.state.list;
    if let Some(index) = list.state.selected() {
        let todo = list.tasks.tasks.iter_mut().nth(index).unwrap().1;
        todo.snoozed = if todo.snoozed.is_some() {
            None
        } else {
            let next_week = next_week();
            Some(next_week)
        };
    }
    // Order snoozed items after non-snoozed items.
    // TODO: make this work again
    // list.tasks.tasks.sort_by_key(|item| item.snoozed.is_some());
}

fn next_week() -> chrono::NaiveDate {
    let now = chrono::Local::now();
    let today = chrono::NaiveDate::from_ymd_opt(now.year(), now.month(), now.day()).unwrap();
    today + chrono::TimeDelta::try_weeks(1).unwrap()
}

fn add(app: &mut App) {
    let list = &mut app.state.list;
    // TODO: make insertion at specific indices work again
    // let index = list.state.selected().unwrap_or(0);
    // *list.state.selected_mut() = Some(index);
    let id = Uuid::new_v4();
    let task = Task {
        id,
        title: String::new(),
        completed: false,
        snoozed: None,
        due_date: None,
    };
    list.tasks.tasks.insert(task.id, task);
    *list.state.selected_mut() = list
        .tasks
        .tasks
        .iter()
        .position(|(&this_id, _)| this_id == id);
    edit(app);
}

fn edit(app: &mut App) {
    let list = &mut app.state.list;
    if let Some(index) = list.state.selected() {
        let task = list.tasks.tasks.iter().nth(index).unwrap().1;
        let text_state = TextState::new()
            .with_value(Cow::Owned(task.title.clone()))
            .with_focus(FocusState::Focused);
        let edit_state = EditState { index, text_state };
        app.state.screen = Screen::Edit(edit_state);
    }
}
