use chrono::Datelike;
use ratatui::widgets::{Block, Borders, List, ListItem};
use ratatui::Frame;

use crate::handle_key_event::Action;
use crate::persist::Task;
use crate::ui_state;
use crate::ui_state::TodoList;

pub fn render(f: &mut Frame, list: &TodoList) {
    let tasks = &list.tasks;
    let items: Vec<_> = tasks.tasks.iter().map(render_task).collect();
    let items = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("List"))
        .highlight_symbol("> ");

    f.render_stateful_widget(items, f.size(), &mut list.state.borrow_mut());
}

fn render_task(s: &Task) -> ListItem<'_> {
    let check = if s.completed.is_some() { 'x' } else { ' ' };
    ListItem::new(format!("[{}] {}", check, s.title.as_str()))
}

pub fn handle_key_event(
    state: &mut ui_state::MainScreenState,
    key_combination: crokey::KeyCombination,
) -> Action {
    let list = &mut state.common_state.list;
    #[allow(clippy::unnested_or_patterns)]
    match key_combination {
        crokey::key!(Esc) | crokey::key!(q) | crokey::key!(Ctrl - c) => Action::Quit,
        crokey::key!(Space) => {
            list.toggle();
            Action::Handled
        }
        crokey::key!(e) => edit(state),
        crokey::key!(S) => {
            snooze(state);
            Action::Handled
        }
        crokey::key!(LEFT) | crokey::key!(H) => {
            list.unselect();
            Action::Handled
        }
        crokey::key!(DOWN) | crokey::key!(J) => {
            list.next();
            Action::Handled
        }
        crokey::key!(UP) | crokey::key!(K) => {
            list.previous();
            Action::Handled
        }
        crokey::key!(A) => add(state),
        crokey::key!(D) => {
            delete(state);
            Action::Handled
        }
        _ => Action::Handled,
    }
}

fn delete(state: &mut ui_state::MainScreenState) {
    let list = &mut state.common_state.list;
    if let Some(index) = list.state.borrow().selected() {
        // Decrement the selected item index by the number of todo
        // items up to it that will be deleted.
        let count = list
            .tasks
            .tasks
            .iter()
            .take(index)
            .filter(|task| task.completed.is_some())
            .count();
        *list.state.borrow_mut().selected_mut() = Some(index - count);
    }
    list.tasks.tasks.retain(|task| task.completed.is_none());
}

fn snooze(state: &mut ui_state::MainScreenState) {
    let list = &mut state.common_state.list;
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

fn add(state: &mut ui_state::MainScreenState) -> Action {
    let list = &mut state.common_state.list;

    let index = list.state.borrow().selected().unwrap_or(0);
    *list.state.borrow_mut().selected_mut() = Some(index);

    let task = Task {
        id: Task::new_id(),
        title: String::new(),
        completed: None,
        snoozed: None,
        due: None,
    };
    list.tasks.tasks.insert(index, task);
    edit(state)
}

fn edit(state: &mut ui_state::MainScreenState) -> Action {
    let list = &mut state.common_state.list;
    if let Some(index) = list.state.borrow().selected() {
        let task = &list.tasks.tasks[index];
        Action::SwitchToEditScreen(index, task.title.clone())
    } else {
        Action::Handled
    }
}
