use ratatui::widgets::{
    Block, Borders, List, ListItem, ListState, StatefulWidget,
};
use std::borrow::Cow;
use std::cell::RefCell;
use std::rc::Rc;

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

fn add(state: Box<State>) -> Box<dyn screen::Screen> {
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

fn edit(mut state: Box<State>) -> Box<dyn screen::Screen> {
    if let Some((id, text)) = {
        let common = &mut state.common.borrow_mut();
        let list = &mut common.list;
        if let Some(id) = list.selected() {
            let title = list.selected_task().unwrap().title.clone();
            let text = tui_prompts::TextState::new()
                .with_value(Cow::Owned(title))
                .with_focus(tui_prompts::FocusState::Focused);
            let text = RefCell::new(text);
            Some((id, text))
        } else {
            None
        }
    } {
        let edit = screen::edit::State {
            common: std::mem::take(&mut state.common),
            id,
            text,
        };
        return Box::new(edit);
    }
    state
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
        mut self: Box<Self>,
        key_combination: crokey::KeyCombination,
    ) -> Box<dyn screen::Screen> {
        {
            let mut common_state = self.common.borrow_mut();
            let list = &mut common_state.list;
            let bindings = crate::keys::bindings();
            match bindings.get(&key_combination) {
                None => {}
                Some(action) => match action {
                    keys::Command::Quit => {
                        return Box::new(screen::quit::State {});
                    }
                    keys::Command::Toggle => {
                        list.toggle();
                    }
                    keys::Command::Edit => {
                        // FIXME: remove this drop
                        std::mem::drop(common_state);
                        return edit(self);
                    }
                    keys::Command::Snooze => {
                        // FIXME: remove this drop
                        std::mem::drop(common_state);
                        snooze(&mut self);
                    }
                    keys::Command::Next => {
                        list.next();
                    }
                    keys::Command::Previous => {
                        list.previous();
                    }
                    keys::Command::MoveUp => {
                        list.move_up();
                    }
                    keys::Command::MoveDown => {
                        list.move_down();
                    }
                    keys::Command::Add => {
                        // FIXME: remove this drop
                        std::mem::drop(common_state);
                        return add(self);
                    }
                    keys::Command::Delete => {
                        // FIXME: remove this drop
                        std::mem::drop(common_state);
                        delete(&mut self);
                    }
                },
            }
        }
        self
    }

    fn render(
        self: Box<Self>,
        frame: &mut ratatui::Frame,
    ) -> Box<dyn screen::Screen> {
        {
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
        self
    }
}
