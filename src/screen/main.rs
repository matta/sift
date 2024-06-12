use ratatui::widgets::{
    Block, Borders, List, ListItem, ListState, StatefulWidget,
};
use std::cell::RefCell;
use std::rc::Rc;

use crate::handle_key_event::Action;
use crate::keys;
use crate::persist::Task;
use crate::screen;
use crate::ui_state::CommonState;

fn render_task(s: &Task) -> ListItem<'_> {
    let check = if s.completed.is_some() { 'x' } else { ' ' };
    ListItem::new(format!("[{}] {}", check, s.title.as_str()))
}

fn delete(state: &mut State) {
    let list = &mut state.common.borrow_mut().list;
    list.delete_selected();
}

fn snooze(state: &mut State) {
    state.common.borrow_mut().list.snooze();
}

fn add(state: &mut State) -> Action {
    {
        let list = &mut state.common.borrow_mut().list;

        let task = Task {
            id: Task::new_id(),
            title: String::new(),
            completed: None,
            snoozed: None,
            due: None,
        };

        list.add_task(task);
    }
    edit(state)
}

fn edit(state: &mut State) -> Action {
    let list = &mut state.common.borrow_mut().list;
    if let Some(selected) = list.selected() {
        let title = list.selected_task().unwrap().title.clone();
        Action::SwitchToEditScreen(selected, title)
    } else {
        Action::Handled
    }
}

pub(crate) struct State {
    pub common: Rc<RefCell<CommonState>>,
    pub list: RefCell<ratatui::widgets::ListState>,
}

impl State {
    pub(crate) fn from_common_state(
        common_state: Rc<RefCell<CommonState>>,
    ) -> Self {
        Self {
            common: common_state,
            list: RefCell::new(ratatui::widgets::ListState::default()),
        }
    }
}

impl Default for State {
    fn default() -> Self {
        State::from_common_state(Rc::new(RefCell::new(CommonState::default())))
    }
}

impl screen::Screen for State {
    fn handle_key_event(
        &mut self,
        key_combination: crokey::KeyCombination,
    ) -> Action {
        let mut common_state = self.common.borrow_mut();
        let list = &mut common_state.list;
        let bindings = crate::keys::bindings();
        match bindings.get(&key_combination) {
            None => Action::Handled,
            Some(action) => match action {
                keys::Command::Quit => Action::Quit,
                keys::Command::Toggle => {
                    list.toggle();
                    Action::Handled
                }
                keys::Command::Edit => {
                    std::mem::drop(common_state);
                    edit(self)
                }
                keys::Command::Snooze => {
                    std::mem::drop(common_state);
                    snooze(self);
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
                keys::Command::Add => {
                    std::mem::drop(common_state);
                    add(self)
                }
                keys::Command::Delete => {
                    std::mem::drop(common_state);
                    delete(self);
                    Action::Handled
                }
            },
        }
    }

    fn render(&self, frame: &mut ratatui::Frame) {
        let list = &self.common.borrow().list;
        let state: &mut ListState = &mut self.list.borrow_mut();
        // Set the list widet's selected state based on the list state.
        state.select(list.index_of_id(list.selected()));

        let items: Vec<_> = list.iter().map(render_task).collect();
        let items = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Tasks"))
            .highlight_symbol("> ");

        items.render(frame.size(), frame.buffer_mut(), state);
    }
}
