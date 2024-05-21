//! Application updater

use std::borrow::Cow;

use chrono::Datelike;
use crokey;
use tui_prompts::FocusState;
use tui_prompts::State as _;
use tui_prompts::TextState;

use crate::persist::Task;
use crate::ui_state;

#[derive(PartialEq, Eq)]
pub(crate) enum Disposition {
    Continue,
    Quit,
}

pub(crate) fn update(
    state: &mut ui_state::State,
    key_event: crossterm::event::KeyEvent,
) -> Disposition {
    match &mut state.current_screen {
        ui_state::Screen::Main => {
            let key_combination: crokey::KeyCombination = key_event.into();
            update_main_screen(key_combination, state)
        }
        ui_state::Screen::Edit(edit_state) => {
            let text_state = &mut edit_state.text_state;
            assert!(text_state.is_focused());
            text_state.handle_key_event(key_event);
            match text_state.status() {
                tui_prompts::Status::Pending => Disposition::Continue,
                tui_prompts::Status::Aborted => {
                    // TODO: When aborting a new item, delete it.
                    state.current_screen = ui_state::Screen::Main;
                    Disposition::Continue
                }
                tui_prompts::Status::Done => {
                    let task = &mut state.list.tasks.tasks[edit_state.index];
                    task.title = text_state.value().into();
                    state.current_screen = ui_state::Screen::Main;
                    Disposition::Continue
                }
            }
        }
    }
}

fn update_main_screen(
    key_combination: crokey::KeyCombination,
    state: &mut ui_state::State,
) -> Disposition {
    #[allow(clippy::unnested_or_patterns)]
    match key_combination {
        crokey::key!(Esc) | crokey::key!(q) => Disposition::Quit,
        crokey::key!(Ctrl - c) => Disposition::Quit,
        crokey::key!(Space) => {
            state.list.toggle();
            Disposition::Continue
        }
        crokey::key!(e) => {
            edit(state);
            Disposition::Continue
        }
        crokey::key!(S) => {
            snooze(state);
            Disposition::Continue
        }
        crokey::key!(LEFT) | crokey::key!(H) => {
            state.list.unselect();
            Disposition::Continue
        }
        crokey::key!(DOWN) | crokey::key!(J) => {
            state.list.next();
            Disposition::Continue
        }
        crokey::key!(UP) | crokey::key!(K) => {
            state.list.previous();
            Disposition::Continue
        }
        crokey::key!(A) => {
            add(state);
            Disposition::Continue
        }
        crokey::key!(D) => {
            delete(state);
            Disposition::Continue
        }
        _ => Disposition::Continue,
    }
}

fn delete(state: &mut ui_state::State) {
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

fn snooze(state: &mut ui_state::State) {
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

fn add(state: &mut ui_state::State) {
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

fn edit(state: &mut ui_state::State) {
    let list = &mut state.list;
    if let Some(index) = list.state.selected() {
        let task = &list.tasks.tasks[index];
        let text_state = TextState::new()
            .with_value(Cow::Owned(task.title.clone()))
            .with_focus(FocusState::Focused);
        let edit_state = ui_state::EditState { index, text_state };
        state.current_screen = ui_state::Screen::Edit(edit_state);
    }
}
