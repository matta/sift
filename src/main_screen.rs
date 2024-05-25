use std::borrow::Cow;
use std::cell::RefCell;

use chrono::Datelike;
use ratatui::Frame;
use ratatui::widgets::{Block, Borders, List, ListItem};
use tui_prompts::{FocusState, TextState};

use crate::handle_key_event::Action;
use crate::persist::Task;
use crate::ui_state;
use crate::ui_state::TodoList;

// struct MainScreen<'a> {
//     state: &'a crate::ui_state::TodoList,
// }

// impl<'a> MainScreen<'a> {
//     pub fn new(state: &'a crate::ui_state::TodoList) -> MainScreen<'a> {
//         MainScreen { state }
//     }
// }

// impl<'a> ratatui::widgets::StatefulWidget for MainScreen<'a> {
//     type State = crate::ui_state::TodoList;
//     fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {}
// }

pub fn render(f: &mut Frame, list: &TodoList) {
    let tasks = &list.tasks;
    let items: Vec<_> = tasks.tasks.iter().map(render_task).collect();
    let items = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("List"))
        .highlight_symbol("> ");

    f.render_stateful_widget(items, f.size(), &mut list.state.borrow_mut());
}

fn render_task(s: &crate::persist::Task) -> ListItem<'_> {
    let check = if s.completed { 'x' } else { ' ' };
    ListItem::new(format!("[{}] {}", check, s.title.as_str()))
}

pub fn handle_key_event(
    state: &mut ui_state::State,
    key_combination: crokey::KeyCombination,
) -> Action {
    #[allow(clippy::unnested_or_patterns)]
    match key_combination {
        crokey::key!(Esc) | crokey::key!(q) | crokey::key!(Ctrl - c) => Action::Quit,
        crokey::key!(Space) => {
            state.list.toggle();
            Action::NoChange
        }
        crokey::key!(e) => {
            edit(state);
            Action::NoChange
        }
        crokey::key!(S) => {
            snooze(state);
            Action::NoChange
        }
        crokey::key!(LEFT) | crokey::key!(H) => {
            state.list.unselect();
            Action::NoChange
        }
        crokey::key!(DOWN) | crokey::key!(J) => {
            state.list.next();
            Action::NoChange
        }
        crokey::key!(UP) | crokey::key!(K) => {
            state.list.previous();
            Action::NoChange
        }
        crokey::key!(A) => {
            add(state);
            Action::NoChange
        }
        crokey::key!(D) => {
            delete(state);
            Action::NoChange
        }
        _ => Action::NoChange,
    }
}

fn delete(state: &mut ui_state::State) {
    let list = &mut state.list;
    if let Some(index) = list.state.borrow().selected() {
        // Decrement the selected item index by the number of todo
        // items up to it that will be deleted.
        let count = list
            .tasks
            .tasks
            .iter()
            .take(index)
            .filter(|task| task.completed)
            .count();
        *list.state.borrow_mut().selected_mut() = Some(index - count);
    }
    list.tasks.tasks.retain(|task| !task.completed);
}

fn snooze(state: &mut ui_state::State) {
    let list = &mut state.list;
    if let Some(index) = list.state.borrow().selected() {
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

    let index = list.state.borrow().selected().unwrap_or(0);
    *list.state.borrow_mut().selected_mut() = Some(index);

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
    if let Some(index) = list.state.borrow().selected() {
        let task = &list.tasks.tasks[index];
        let text_state = TextState::new()
            .with_value(Cow::Owned(task.title.clone()))
            .with_focus(FocusState::Focused);
        let edit_state = ui_state::EditState {
            index,
            text_state: RefCell::new(text_state),
        };
        state.current_screen = ui_state::Screen::Edit(edit_state);
    }
}
