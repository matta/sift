use std::borrow::Cow;
use std::cell::RefCell;

use ratatui::widgets::{
    Block, Borders, List, ListItem, ListState, StatefulWidget,
};

use crate::persist::Task;
use crate::screen::Screen;
use crate::ui_state::CommonState;
use crate::{keys, screen};

fn render_task(s: &Task) -> ListItem<'_> {
    let check = if s.completed.is_some() { 'x' } else { ' ' };
    ListItem::new(format!("[{}] {}", check, s.title.as_str()))
}

fn add(common_state: &mut CommonState, state: Box<State>) -> Box<dyn Screen> {
    {
        let task = Task {
            id: Task::new_id(),
            title: String::new(),
            completed: None,
            snoozed: None,
            due: None,
        };

        common_state.list.add_task(task);
    }
    edit(common_state, state)
}

fn edit(common_state: &mut CommonState, state: Box<State>) -> Box<dyn Screen> {
    if let Some((id, text)) = {
        let list = &mut common_state.list;
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
        let edit = screen::edit::State::new(id, text);
        return Box::new(edit);
    }
    state
}

#[derive(Default)]
pub(crate) struct State {
    list: RefCell<ratatui::widgets::ListState>,
}

impl State {
    pub(crate) fn new() -> Self {
        Self::default()
    }
}

impl screen::Screen for State {
    fn handle_key_event(
        self: Box<Self>,
        common_state: &mut CommonState,
        key_combination: crokey::KeyCombination,
    ) -> Box<dyn Screen> {
        {
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
                        return edit(common_state, self);
                    }
                    keys::Command::Snooze => {
                        list.snooze();
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
                        return add(common_state, self);
                    }
                    keys::Command::Delete => {
                        common_state.list.delete_selected();
                    }
                },
            }
        }
        self
    }

    fn render(
        self: Box<Self>,
        common_state: &mut CommonState,
        frame: &mut ratatui::Frame,
    ) -> Box<dyn Screen> {
        {
            let list = &common_state.list;
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
