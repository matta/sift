use std::borrow::Cow;

use std::cell::RefCell;
use tui_prompts::State as _;
use tui_prompts::TextPrompt;

use crate::screen;
use crate::ui_state::CommonState;

pub(crate) struct State {
    id: uuid::Uuid,
    // TODO: in upstream make the 'static workaround used here more
    // discoverable.  See
    // https://github.com/rhysd/tui-textarea/issues/46
    text: RefCell<tui_prompts::TextState<'static>>,
}

impl State {
    pub(crate) fn new(
        id: uuid::Uuid,
        text: RefCell<tui_prompts::prelude::TextState<'static>>,
    ) -> Self {
        Self { id, text }
    }
}

impl screen::Screen for State {
    type Context = CommonState;

    fn handle_key_event(
        self: Box<Self>,
        context: &mut Self::Context,
        key_combination: crokey::KeyCombination,
    ) -> Box<dyn screen::Screen<Context = Self::Context>> {
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
                Box::new(screen::main::State::new())
            }
            tui_prompts::Status::Done => {
                {
                    context.list.set_title(self.id, text_state.value().into());
                }
                Box::new(screen::main::State::new())
            }
        }
    }

    fn render(
        self: Box<Self>,
        _conext: &mut Self::Context,
        frame: &mut ratatui::Frame,
    ) -> Box<dyn screen::Screen<Context = Self::Context>> {
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