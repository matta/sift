use std::any::Any;

pub mod edit;
pub mod main;
pub mod quit;

pub trait Screen: Any {
    #[must_use]
    fn handle_key_event(
        self: Box<Self>,
        key_combination: crokey::KeyCombination,
    ) -> Box<dyn Screen>;

    #[must_use]
    fn render(self: Box<Self>, frame: &mut ratatui::Frame) -> Box<dyn Screen>;

    // FIXME: replace this with a back channel to the event queue logic?
    fn should_quit(&self) -> bool {
        false
    }
}
