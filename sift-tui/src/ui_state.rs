/*!
Code for managing the displayed state of the application.

The `State` struct contains the application's state.  It is the
central data structure for the application.
*/

use std::path::Path;

use anyhow::Result;
use sift_persist::MemoryStore;

use crate::screen::{self, Screen};

pub(crate) struct State {
    // FIXME: make non-public
    pub common_state: sift_state::State,
    pub current_screen: Option<Box<dyn Screen>>,
}

impl Default for State {
    fn default() -> Self {
        let current_screen = Some(Box::<dyn Screen>::from(
            Box::new(screen::main::State::new()),
        ));
        State {
            common_state: sift_state::State::default(),
            current_screen,
        }
    }
}

impl State {
    pub fn new() -> State {
        State::default()
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        self.common_state.store.save(path)
    }

    pub fn load(path: &Path) -> Result<State> {
        let store = MemoryStore::load(path)?;
        let common_state = sift_state::State::new(store);
        let state = State {
            common_state,
            ..Default::default()
        };
        Ok(state)
    }
}
