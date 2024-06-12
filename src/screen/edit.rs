use std::borrow::Cow;
use std::rc::Rc;

use std::cell::RefCell;
use tui_prompts::State as _;
use tui_prompts::TextPrompt;

use crate::handle_key_event::Action;
use crate::screen;
use crate::ui_state::CommonState;

pub(crate) struct State {
    pub common: Rc<RefCell<CommonState>>,
    pub id: uuid::Uuid,
    // TODO: in upstream make the 'static workaround used here more
    // discoverable.  See
    // https://github.com/rhysd/tui-textarea/issues/46
    pub text: RefCell<tui_prompts::TextState<'static>>,
}

impl screen::Screen for State {
    fn handle_key_event(
        &mut self,
        key_combination: crokey::KeyCombination,
    ) -> Action {
        let mut text_state = self.text.borrow_mut();
        assert!(text_state.is_focused());
        let key_event: crossterm::event::KeyEvent = key_combination.into();
        text_state.handle_key_event(key_event);
        match text_state.status() {
            tui_prompts::Status::Pending => Action::Handled,
            tui_prompts::Status::Aborted => {
                // TODO: When aborting a new item, delete it.
                Action::SwitchToMainScreen
            }
            tui_prompts::Status::Done => {
                let common_state = &mut self.common.borrow_mut();
                common_state
                    .list
                    .set_title(self.id, text_state.value().into());
                Action::SwitchToMainScreen
            }
        }
    }

    fn render(&self, frame: &mut ratatui::Frame) {
        let prompt = TextPrompt::new(Cow::Borrowed("edit"));
        frame.render_stateful_widget(
            prompt,
            frame.size(),
            &mut self.text.borrow_mut(),
        );
        let (x, y) = self.text.borrow().cursor();
        frame.set_cursor(x, y);
    }
}
