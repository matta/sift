//! Application updater

use std::borrow::Cow;

use chrono::Datelike;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tui_prompts::FocusState;
use tui_prompts::State as _;
use tui_prompts::TextState;

use crate::persist::Task;
use crate::state;

#[derive(PartialEq, Eq)]
pub(crate) enum Disposition {
    Continue,
    Quit,
}

pub(crate) fn update(state: &mut state::State, key_event: KeyEvent) -> Disposition {
    match &mut state.current_screen {
        state::Screen::Main => match key_event.code {
            KeyCode::Esc | KeyCode::Char('q') => Disposition::Quit,
            KeyCode::Char('c' | 'C') if key_event.modifiers == KeyModifiers::CONTROL => {
                Disposition::Quit
            }
            KeyCode::Char(' ') => {
                state.list.toggle();
                Disposition::Continue
            }
            KeyCode::Char('e') => {
                edit(state);
                Disposition::Continue
            }
            KeyCode::Char('s') => {
                snooze(state);
                Disposition::Continue
            }
            KeyCode::Left | KeyCode::Char('h') => {
                state.list.unselect();
                Disposition::Continue
            }
            KeyCode::Down | KeyCode::Char('j') => {
                state.list.next();
                Disposition::Continue
            }
            KeyCode::Up | KeyCode::Char('k') => {
                state.list.previous();
                Disposition::Continue
            }
            KeyCode::Char('a') => {
                add(state);
                Disposition::Continue
            }
            KeyCode::Char('D') => {
                delete(state);
                Disposition::Continue
            }
            _ => Disposition::Continue,
        },
        state::Screen::Edit(edit_state) => {
            let text_state = &mut edit_state.text_state;
            assert!(text_state.is_focused());
            text_state.handle_key_event(key_event);
            match text_state.status() {
                tui_prompts::Status::Pending => Disposition::Continue,
                tui_prompts::Status::Aborted => {
                    // TODO: When aborting a new item, delete it.
                    state.current_screen = state::Screen::Main;
                    Disposition::Continue
                }
                tui_prompts::Status::Done => {
                    let task = &mut state.list.tasks.tasks[edit_state.index];
                    task.title = text_state.value().into();
                    state.current_screen = state::Screen::Main;
                    Disposition::Continue
                }
            }
        }
    }
}

fn delete(state: &mut state::State) {
    let list = &mut state.list;
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

fn snooze(state: &mut state::State) {
    let list = &mut state.list;
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

fn add(state: &mut state::State) {
    let list = &mut state.list;

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
    edit(state);
}

fn edit(state: &mut state::State) {
    let list = &mut state.list;
    if let Some(index) = list.state.selected() {
        let task = &list.tasks.tasks[index];
        let text_state = TextState::new()
            .with_value(Cow::Owned(task.title.clone()))
            .with_focus(FocusState::Focused);
        let edit_state = state::EditState { index, text_state };
        state.current_screen = state::Screen::Edit(edit_state);
    }
}
