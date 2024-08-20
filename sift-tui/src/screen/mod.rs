pub mod edit;
pub mod main;
pub mod quit;

pub trait Screen {
    // TODO: return an Option<Box<dyn Screen>> instead.
    #[must_use]
    fn handle_key_event(
        self: Box<Self>,
        context: &mut sift_state::State,
        key_combination: crokey::KeyCombination,
    ) -> Box<dyn Screen>;

    fn render(&self, conext: &mut sift_state::State, frame: &mut ratatui::Frame);

    // FIXME: replace this with a back channel to the event queue logic?
    // ...at which point should handle_key_event return a Box<dyn Screen>
    // ...or should it return an enum with a NewScreen variant and another
    // ...for the quit case?
    fn should_quit(&self, context: &mut sift_state::State) -> bool {
        _ = context;
        false
    }
}
