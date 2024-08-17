use std::borrow::Cow;
use std::cell::RefCell;

use ratatui::crossterm;
use sift_core::persist::{Store, TaskId, Transaction};
use tui_prompts::{State as _, TextPrompt};

use crate::screen;
use crate::ui_state::CommonState;

pub(crate) struct State {
    id: TaskId,
    // TODO: in upstream make the 'static workaround used here more
    // discoverable.  See
    // https://github.com/rhysd/tui-textarea/issues/46
    text: RefCell<tui_prompts::TextState<'static>>,
}

impl State {
    pub(crate) fn new(id: TaskId, text: RefCell<tui_prompts::prelude::TextState<'static>>) -> Self {
        Self { id, text }
    }

    fn do_handle_key_event(
        &mut self,
        context: &mut CommonState,
        key_combination: crokey::KeyCombination,
    ) -> Option<Box<dyn screen::Screen>> {
        let mut text_state = self.text.borrow_mut();
        assert!(text_state.is_focused());
        let key_event: crossterm::event::KeyEvent = key_combination.into();
        text_state.handle_key_event(key_event);
        match text_state.status() {
            tui_prompts::Status::Pending => None,
            tui_prompts::Status::Aborted => {
                // TODO: When aborting a new item, delete it.
                Some(Box::new(screen::main::State::new()))
            }
            tui_prompts::Status::Done => {
                let title = text_state.value();
                let id = &self.id;
                context
                    .store
                    .with_transaction(|txn| set_title(txn, id, title))
                    .expect("TODO: handle error");
                Some(Box::new(screen::main::State::new()))
            }
        }
    }
}

fn set_title(txn: &mut dyn Transaction, id: &TaskId, title: &str) -> Result<(), anyhow::Error> {
    let mut task = txn.get_task(id).unwrap();
    task.set_title(title.to_string());
    txn.put_task(&task)
}

impl screen::Screen for State {
    fn handle_key_event(
        mut self: Box<Self>,
        context: &mut CommonState,
        key_combination: crokey::KeyCombination,
    ) -> Box<dyn screen::Screen> {
        if let Some(screen) = self.do_handle_key_event(context, key_combination) {
            screen
        } else {
            self
        }
    }

    fn render(&self, _conext: &mut CommonState, frame: &mut ratatui::Frame) {
        let prompt = TextPrompt::new(Cow::Borrowed("edit"));
        frame.render_stateful_widget(prompt, frame.area(), &mut self.text.borrow_mut());
        let (x, y) = self.text.borrow().cursor();
        frame.set_cursor(x, y);
    }
}
