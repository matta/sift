use crate::ui_state::CommonState;

pub mod edit;
pub mod main;
pub mod quit;

pub trait Screen {
    // TODO: return an Option<Box<dyn Screen>> instead.
    #[must_use]
    fn handle_key_event(
        self: Box<Self>,
        context: &mut CommonState,
        key_combination: crokey::KeyCombination,
    ) -> Box<dyn Screen>;

    fn render(self: &Self, conext: &mut CommonState, frame: &mut ratatui::Frame);

    // FIXME: replace this with a back channel to the event queue logic?
    fn should_quit(&self, context: &mut CommonState) -> bool {
        _ = context;
        false
    }
}
