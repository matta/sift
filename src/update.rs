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
    NoChange,
    AcceptTaskEdit(usize, String),
    SwitchToMainScreen,
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
        ui_state::Screen::Edit(edit_state) => update_edit_screen(edit_state, key_event),
    }
}

fn update_edit_screen(
    edit_state: &mut ui_state::EditState,
    key_event: crossterm::event::KeyEvent,
) -> Disposition {
    let text_state = &mut edit_state.text_state;
    assert!(text_state.is_focused());
    text_state.handle_key_event(key_event);
    match text_state.status() {
        tui_prompts::Status::Pending => Disposition::NoChange,
        tui_prompts::Status::Aborted => {
            // TODO: When aborting a new item, delete it.
            Disposition::SwitchToMainScreen
        }
        tui_prompts::Status::Done => {
            Disposition::AcceptTaskEdit(edit_state.index, text_state.value().into())
        }
    }
}

fn update_main_screen(
    key_combination: crokey::KeyCombination,
    state: &mut ui_state::State,
) -> Disposition {
    #[allow(clippy::unnested_or_patterns)]
    match key_combination {
        crokey::key!(Esc) | crokey::key!(q) | crokey::key!(Ctrl - c) => Disposition::Quit,
        crokey::key!(Space) => {
            state.list.toggle();
            Disposition::NoChange
        }
        crokey::key!(e) => {
            edit(state);
            Disposition::NoChange
        }
        crokey::key!(S) => {
            snooze(state);
            Disposition::NoChange
        }
        crokey::key!(LEFT) | crokey::key!(H) => {
            state.list.unselect();
            Disposition::NoChange
        }
        crokey::key!(DOWN) | crokey::key!(J) => {
            state.list.next();
            Disposition::NoChange
        }
        crokey::key!(UP) | crokey::key!(K) => {
            state.list.previous();
            Disposition::NoChange
        }
        crokey::key!(A) => {
            add(state);
            Disposition::NoChange
        }
        crokey::key!(D) => {
            delete(state);
            Disposition::NoChange
        }
        _ => Disposition::NoChange,
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
