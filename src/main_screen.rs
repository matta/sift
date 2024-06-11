use chrono::Datelike;
use ratatui::widgets::{Block, Borders, List, ListItem, StatefulWidget};
use ratatui::Frame;

use crate::handle_key_event::Action;
use crate::persist::Task;
use crate::ui_state::TodoList;
use crate::{keys, ui_state};

pub fn render(f: &mut Frame, list: &TodoList) {
    let tasks = &list.tasks;
    let items: Vec<_> = tasks.tasks.iter().filter_map(render_task).collect();
    let items = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Tasks"))
        .highlight_symbol("> ");

    items.render(f.size(), f.buffer_mut(), &mut list.list_state.borrow_mut());
}

fn render_task(s: &Task) -> Option<ListItem<'_>> {
    if matches!(s.snoozed, Some(date) if date > today()) {
        return None;
    }
    let check = if s.completed.is_some() { 'x' } else { ' ' };
    let item = ListItem::new(format!("[{}] {}", check, s.title.as_str()));
    Some(item)
}

pub fn handle_key_event(
    state: &mut ui_state::MainScreenState,
    key_combination: crokey::KeyCombination,
) -> Action {
    let list = &mut state.common_state.list;
    let bindings = crate::keys::bindings();
    match bindings.get(&key_combination) {
        None => Action::Handled,
        Some(action) => match action {
            keys::Command::Quit => Action::Quit,
            keys::Command::Toggle => {
                list.toggle();
                Action::Handled
            }
            keys::Command::Edit => edit(state),
            keys::Command::Snooze => {
                snooze(state);
                Action::Handled
            }
            keys::Command::Next => {
                list.next();
                Action::Handled
            }
            keys::Command::Previous => {
                list.previous();
                Action::Handled
            }
            keys::Command::MoveUp => {
                list.move_up();
                Action::Handled
            }
            keys::Command::MoveDown => {
                list.move_down();
                Action::Handled
            }
            keys::Command::Add => add(state),
            keys::Command::Delete => {
                delete(state);
                Action::Handled
            }
        },
    }
}

fn delete(state: &mut ui_state::MainScreenState) {
    let list = &mut state.common_state.list;
    if let Some(index) = list.list_state.borrow().selected() {
        // Decrement the selected item index by the number of todo
        // items up to it that will be deleted.
        let count = list
            .tasks
            .tasks
            .iter()
            .take(index)
            .filter(|task| task.completed.is_some())
            .count();
        *list.list_state.borrow_mut().selected_mut() = Some(index - count);
    }
    list.tasks.tasks.retain(|task| task.completed.is_none());
}

fn snooze(state: &mut ui_state::MainScreenState) {
    let list = &mut state.common_state.list;
    if let Some(index) = list.list_state.borrow().selected() {
        let task = &mut list.tasks.tasks[index];
        task.snoozed = if task.snoozed.is_some() {
            None
        } else {
            let next_week = next_week();
            Some(next_week)
        };
    }
    // Order snoozed items after non-snoozed items.  Keep the current selection.
    //
    // Note: this is a stable sort.
    // Note: false sorts before true.
    list.tasks.tasks.sort_by_key(|task| task.snoozed.is_some());
}

fn today() -> chrono::NaiveDate {
    let now = chrono::Local::now();
    chrono::NaiveDate::from_ymd_opt(now.year(), now.month(), now.day()).unwrap()
}

fn next_week() -> chrono::NaiveDate {
    today() + chrono::TimeDelta::try_weeks(1).unwrap()
}

fn add(state: &mut ui_state::MainScreenState) -> Action {
    let list = &mut state.common_state.list;

    let index = list.list_state.borrow().selected().unwrap_or(0);
    *list.list_state.borrow_mut().selected_mut() = Some(index);

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
    if let Some(index) = list.list_state.borrow().selected() {
        let task = &list.tasks.tasks[index];
        Action::SwitchToEditScreen(index, task.title.clone())
    } else {
        Action::Handled
    }
}
