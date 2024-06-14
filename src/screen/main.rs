use std::borrow::Cow;
use std::cell::RefCell;

use ratatui::widgets::{Block, Borders, List, ListItem, ListState, StatefulWidget};

use crate::persist::{Store, Task};
use crate::screen::Screen;
use crate::ui_state::CommonState;
use crate::{keys, screen};

fn render_task(s: &Task) -> ListItem<'_> {
    let check = if s.completed().is_some() { 'x' } else { ' ' };
    ListItem::new(format!("[{}] {}", check, s.title()))
}

fn add(common_state: &mut CommonState, state: Box<State>) -> Box<dyn Screen> {
    // FIXME: make generating new tasks less cumbersome
    // FIXME: handle error
    let task = Task::new(Task::new_id(), String::new(), None, None, None);
    common_state
        .store
        .insert(common_state.selected.as_ref(), &task)
        .expect("FIXME: handle error");
    common_state.selected = Some(task.id());
    edit(common_state, state)
}

fn edit(common_state: &mut CommonState, state: Box<State>) -> Box<dyn Screen> {
    if let Some((id, text)) = {
        if let Some(id) = common_state.selected {
            let title = common_state.store.get(&id).unwrap().title().into();
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
            let bindings = crate::keys::bindings();
            match bindings.get(&key_combination) {
                None => {}
                Some(action) => match action {
                    keys::Command::Quit => {
                        return Box::new(screen::quit::State {});
                    }
                    keys::Command::Toggle => {
                        common_state.toggle();
                    }
                    keys::Command::Edit => {
                        return edit(common_state, self);
                    }
                    keys::Command::Snooze => {
                        common_state.snooze();
                    }
                    keys::Command::Next => {
                        common_state.next();
                    }
                    keys::Command::Previous => {
                        common_state.previous();
                    }
                    keys::Command::MoveUp => {
                        common_state.move_up();
                    }
                    keys::Command::MoveDown => {
                        common_state.move_down();
                    }
                    keys::Command::Add => {
                        return add(common_state, self);
                    }
                    keys::Command::Delete => {
                        common_state.delete_selected();
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
            // Set the list widet's selected state based on the list state.
            let state: &mut ListState = &mut self.list.borrow_mut();
            state.select(common_state.index_of_id(common_state.selected));

            let items: Vec<_> = common_state
                .iter_tasks_for_display()
                .map(render_task)
                .collect();
            let items = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Tasks"))
                .highlight_symbol("> ");

            items.render(frame.size(), frame.buffer_mut(), state);
        }
        self
    }
}
