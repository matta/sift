use ratatui::text::Line;

use super::Screen;
use crate::ui_state::CommonState;

pub(crate) struct State {}

impl Screen for State {
    fn handle_key_event(
        self: Box<Self>,
        _context: &mut CommonState,
        _key_combination: crokey::KeyCombination,
    ) -> Box<dyn Screen> {
        self
    }

    fn render(&self, _conext: &mut CommonState, frame: &mut ratatui::Frame) {
        frame.render_widget(Line::from("quitting..."), frame.area());
    }

    fn should_quit(&self, _context: &mut CommonState) -> bool {
        true
    }
}
