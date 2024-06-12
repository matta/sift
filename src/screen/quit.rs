use ratatui::text::Line;

use crate::ui_state::CommonState;

use super::Screen;

pub(crate) struct State {}

impl Screen for State {
    fn handle_key_event(
        self: Box<Self>,
        _context: &mut CommonState,
        _key_combination: crokey::KeyCombination,
    ) -> Box<dyn Screen> {
        self
    }

    fn render(
        self: Box<Self>,
        _conext: &mut CommonState,
        frame: &mut ratatui::Frame,
    ) -> Box<dyn Screen> {
        frame.render_widget(Line::from("quitting..."), frame.size());
        self
    }

    fn should_quit(&self, _context: &mut CommonState) -> bool {
        true
    }
}
