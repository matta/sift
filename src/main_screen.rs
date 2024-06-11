use chrono::Datelike;
use ratatui::widgets::{
    Block, Borders, List, ListItem, ListState, StatefulWidget,
};
use ratatui::Frame;

use crate::handle_key_event::Action;
use crate::persist::Task;
use crate::ui_state::TodoList;
use crate::{keys, ui_state};

pub fn render(f: &mut Frame, list: &TodoList, state: &mut ListState) {
    let items: Vec<_> = list.iter().filter_map(render_task).collect();
    let items = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Tasks"))
        .highlight_symbol("> ");

    items.render(f.size(), f.buffer_mut(), state);
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
    list.delete_selected();
}

fn snooze(state: &mut ui_state::MainScreenState) {
    state.common_state.list.snooze();
}

// TODO: move this elsewhere
pub(crate) fn today() -> chrono::NaiveDate {
    let now = chrono::Local::now();
    chrono::NaiveDate::from_ymd_opt(now.year(), now.month(), now.day()).unwrap()
}

fn add(state: &mut ui_state::MainScreenState) -> Action {
    let list = &mut state.common_state.list;

    let task = Task {
        id: Task::new_id(),
        title: String::new(),
        completed: None,
        snoozed: None,
        due: None,
    };

    list.add_task(task);
    edit(state)
}

fn edit(state: &mut ui_state::MainScreenState) -> Action {
    let list = &mut state.common_state.list;
    if let Some(selected) = list.selected() {
        let title = list.selected_task().unwrap().title.clone();
        Action::SwitchToEditScreen(selected, title)
    } else {
        Action::Handled
    }
}
