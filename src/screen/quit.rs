use ratatui::text::Line;

use crate::screen;

pub(crate) struct State {}

impl screen::Screen for State {
    fn handle_key_event(
        self: Box<Self>,
        _key_combination: crokey::KeyCombination,
    ) -> Box<dyn screen::Screen> {
        self
    }

    fn render(
        self: Box<Self>,
        frame: &mut ratatui::Frame,
    ) -> Box<dyn screen::Screen> {
        frame.render_widget(Line::from("quitting..."), frame.size());
        self
    }

    fn should_quit(&self) -> bool {
        true
    }
}
