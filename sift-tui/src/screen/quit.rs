use ratatui::text::Line;

use super::Screen;

pub(crate) struct State {}

impl Screen for State {
    fn handle_key_event(
        self: Box<Self>,
        _context: &mut sift_state::State,
        _key_combination: crokey::KeyCombination,
    ) -> Box<dyn Screen> {
        self
    }

    fn render(&self, _conext: &mut sift_state::State, frame: &mut ratatui::Frame) {
        frame.render_widget(Line::from("quitting..."), frame.area());
    }

    fn should_quit(&self, _context: &mut sift_state::State) -> bool {
        true
    }
}
