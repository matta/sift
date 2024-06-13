use crate::ui_state::CommonState;

pub mod edit;
pub mod main;
pub mod quit;

pub trait Screen {
    #[must_use]
    fn handle_key_event(
        self: Box<Self>,
        context: &mut CommonState,
        key_combination: crokey::KeyCombination,
    ) -> Box<dyn Screen>;

    #[must_use]
    fn render(
        self: Box<Self>,
        conext: &mut CommonState,
        frame: &mut ratatui::Frame,
    ) -> Box<dyn Screen>;

    // FIXME: replace this with a back channel to the event queue logic?
    fn should_quit(&self, context: &mut CommonState) -> bool {
        _ = context;
        false
    }
}
