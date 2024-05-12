use std::borrow::Cow;

use chrono::Datelike;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tui_prompts::FocusState;
use tui_prompts::State as _;
use tui_prompts::TextState;

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
                    let task = &mut app.state.list.tasks.tasks[edit_state.index];
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
