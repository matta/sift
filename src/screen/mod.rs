pub mod edit;
pub mod main;
pub mod quit;

pub trait Screen {
    type Context;

    #[must_use]
    fn handle_key_event(
        self: Box<Self>,
        context: &mut Self::Context,
        key_combination: crokey::KeyCombination,
    ) -> Box<dyn Screen<Context = Self::Context>>;

    #[must_use]
    fn render(
        self: Box<Self>,
        conext: &mut Self::Context,
        frame: &mut ratatui::Frame,
    ) -> Box<dyn Screen<Context = Self::Context>>;

    // FIXME: replace this with a back channel to the event queue logic?
    #[deprecated]
    fn should_quit(&self, _context: &mut Self::Context) -> bool {
        false
    }
}
