//! Application updater

use std::borrow::Cow;

use chrono::Datelike;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tui_prompts::FocusState;
use tui_prompts::State as _;
use tui_prompts::TextState;

use crate::app::{App, EditState, Screen};
use crate::persist::Task;

#[derive(PartialEq, Eq)]
pub(crate) enum Disposition {
    Continue,
    Quit,
}

pub(crate) fn update(app: &mut App, key_event: KeyEvent) -> Disposition {
    match &mut app.state.screen {
        Screen::Main => match key_event.code {
            KeyCode::Esc | KeyCode::Char('q') => Disposition::Quit,
            KeyCode::Char('c' | 'C') if key_event.modifiers == KeyModifiers::CONTROL => {
                Disposition::Quit
            }
            KeyCode::Char(' ') => {
                app.state.list.toggle();
                Disposition::Continue
            }
            KeyCode::Char('e') => {
                edit(app);
                Disposition::Continue
            }
            KeyCode::Char('s') => {
                snooze(app);
                Disposition::Continue
            }
            KeyCode::Left | KeyCode::Char('h') => {
                app.state.list.unselect();
                Disposition::Continue
            }
            KeyCode::Down | KeyCode::Char('j') => {
                app.state.list.next();
                Disposition::Continue
            }
            KeyCode::Up | KeyCode::Char('k') => {
                app.state.list.previous();
                Disposition::Continue
            }
            KeyCode::Char('a') => {
                add(app);
                Disposition::Continue
            }
            KeyCode::Char('D') => {
                delete(app);
                Disposition::Continue
            }
            _ => Disposition::Continue,
        },
        Screen::Edit(edit_state) => {
            let text_state = &mut edit_state.text_state;
            assert!(text_state.is_focused());
            text_state.handle_key_event(key_event);
            match text_state.status() {
                tui_prompts::Status::Pending => Disposition::Continue,
                tui_prompts::Status::Aborted => {
                    // TODO: When aborting a new item, delete it.
                    app.state.screen = Screen::Main;
                    Disposition::Continue
                }
                tui_prompts::Status::Done => {
                    let task = &mut app.state.list.tasks.tasks[edit_state.index];
                    task.title = text_state.value().into();
                    app.state.screen = Screen::Main;
                    Disposition::Continue
                }
            }
        }
    }
}

fn delete(app: &mut App) {
    let list = &mut app.state.list;
    if let Some(index) = list.state.selected() {
        // Decrement the selected item index by the number of todo
        // items up to it that will be deleted.
        let count = list
            .tasks
            .tasks
            .iter()
            .take(index)
            .filter(|task| task.completed)
            .count();
        *list.state.selected_mut() = Some(index - count);
    }
    list.tasks.tasks.retain(|task| !task.completed);
}

fn snooze(app: &mut App) {
    let list = &mut app.state.list;
    if let Some(index) = list.state.selected() {
        let task = &mut list.tasks.tasks[index];
        task.snoozed = if task.snoozed.is_some() {
            None
        } else {
            let next_week = next_week();
            Some(next_week)
        };
    }
    // Order snoozed items after non-snoozed items.
    // Boolean false comes before true.
    // Note: this is a stable sort.
    list.tasks.tasks.sort_by_key(|task| task.snoozed.is_some());
}

fn next_week() -> chrono::NaiveDate {
    let now = chrono::Local::now();
    let today = chrono::NaiveDate::from_ymd_opt(now.year(), now.month(), now.day()).unwrap();
    today + chrono::TimeDelta::try_weeks(1).unwrap()
}

fn add(app: &mut App) {
    let list = &mut app.state.list;

    let index = list.state.selected().unwrap_or(0);
    *list.state.selected_mut() = Some(index);

    let task = Task {
        id: Task::new_id(),
        title: String::new(),
        completed: false,
        snoozed: None,
        due_date: None,
    };
    list.tasks.tasks.insert(index, task);
    edit(app);
}

fn edit(app: &mut App) {
    let list = &mut app.state.list;
    if let Some(index) = list.state.selected() {
        let task = &list.tasks.tasks[index];
        let text_state = TextState::new()
            .with_value(Cow::Owned(task.title.clone()))
            .with_focus(FocusState::Focused);
        let edit_state = EditState { index, text_state };
        app.state.screen = Screen::Edit(edit_state);
    }
}
