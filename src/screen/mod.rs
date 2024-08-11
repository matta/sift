use xilem::AnyWidgetView;

use crate::ui_state::{self, CommonState};

pub mod edit;
pub mod main;
pub mod quit;

pub trait Screen {
    fn render(&self, state: &mut CommonState) -> Box<AnyWidgetView<ui_state::State, ()>>;
}
