use std::borrow::Cow;
use std::rc::Rc;

use std::cell::RefCell;
use tui_prompts::State as _;
use tui_prompts::TextPrompt;

use crate::screen;
use crate::ui_state::CommonState;

pub(crate) struct State {
    // TODO: move CommonState to the State trait and take it as args to the
    // trait methods.
    common: Rc<RefCell<CommonState>>,
    id: uuid::Uuid,
    // TODO: in upstream make the 'static workaround used here more
    // discoverable.  See
    // https://github.com/rhysd/tui-textarea/issues/46
    text: RefCell<tui_prompts::TextState<'static>>,
}

impl State {
    pub(crate) fn new(
        common: Rc<RefCell<CommonState>>,
        id: uuid::Uuid,
        text: RefCell<tui_prompts::prelude::TextState<'static>>,
    ) -> Self {
        Self { common, id, text }
    }
}

impl screen::Screen for State {
    fn handle_key_event(
        self: Box<Self>,
        key_combination: crokey::KeyCombination,
    ) -> Box<dyn screen::Screen> {
        let mut text_state = self.text.borrow_mut();
        assert!(text_state.is_focused());
        let key_event: crossterm::event::KeyEvent = key_combination.into();
        text_state.handle_key_event(key_event);
        match text_state.status() {
            tui_prompts::Status::Pending => {
                // FIXME: having to do these drops sucks. Restructure somehow?
                // Stems from having to return self.
                std::mem::drop(text_state);
                self
            }
            tui_prompts::Status::Aborted => {
                // TODO: When aborting a new item, delete it.
                Box::new(screen::main::State::from_common_state(self.common))
            }
            tui_prompts::Status::Done => {
                {
                    let common_state = &mut self.common.borrow_mut();
                    common_state
                        .list
                        .set_title(self.id, text_state.value().into());
                }
                Box::new(screen::main::State::from_common_state(self.common))
            }
        }
    }

    fn render(
        self: Box<Self>,
        frame: &mut ratatui::Frame,
    ) -> Box<dyn screen::Screen> {
        let prompt = TextPrompt::new(Cow::Borrowed("edit"));
        frame.render_stateful_widget(
            prompt,
            frame.size(),
            &mut self.text.borrow_mut(),
        );
        let (x, y) = self.text.borrow().cursor();
        frame.set_cursor(x, y);
        self
    }
}
