use ratatui::text::Line;

use crate::{screen, ui_state};

pub(crate) struct State {}

impl screen::Screen for State {
    type Context = ui_state::CommonState;

    fn handle_key_event(
        self: Box<Self>,
        _context: &mut Self::Context,
        _key_combination: crokey::KeyCombination,
    ) -> Box<dyn screen::Screen<Context = Self::Context>> {
        self
    }

    fn render(
        self: Box<Self>,
        _conext: &mut Self::Context,
        frame: &mut ratatui::Frame,
    ) -> Box<ui_state::Screen> {
        frame.render_widget(Line::from("quitting..."), frame.size());
        self
    }

    fn should_quit(&self, _context: &mut Self::Context) -> bool {
        true
    }
}
