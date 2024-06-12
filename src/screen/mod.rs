use crate::handle_key_event::Action;

pub mod edit;
pub mod main;

pub trait Screen {
    fn handle_key_event(
        &mut self,
        key_combination: crokey::KeyCombination,
    ) -> Action;
    fn render(&self, frame: &mut ratatui::Frame);
}
