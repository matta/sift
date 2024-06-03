/*!
Code for managing the displayed state of the application.

The `State` struct contains the application's state.  It is the
central data structure for the application.
*/

use std::{
    cell::RefCell,
    fs::File,
    io::{Read, Write},
    path::Path,
};

use anyhow::Result;

use crate::persist;

#[derive(Default)]
pub(crate) struct State {
    pub current_screen: Screen,
}

#[derive(Default)]
pub(crate) struct CommonState {
    pub list: TodoList,
}

pub(crate) enum Screen {
    Main(MainScreenState),
    Edit(EditScreenState),
}

impl Default for Screen {
    fn default() -> Self {
        Screen::Main(MainScreenState::default())
    }
}

impl Screen {
    pub(crate) fn mut_common_state(&mut self) -> &mut CommonState {
        match self {
            Screen::Main(s) => &mut s.common_state,
            Screen::Edit(s) => &mut s.common_state,
        }
    }

    pub(crate) fn take_common_state(&mut self) -> CommonState {
        std::mem::take(self.mut_common_state())
    }
}

#[derive(Default)]
pub(crate) struct MainScreenState {
    pub common_state: CommonState,
}

pub(crate) struct EditScreenState {
    pub common_state: CommonState,
    pub index: usize,
    // TODO: in upstream make the 'static workaround used here more
    // discoverable.  See
    // https://github.com/rhysd/tui-textarea/issues/46
    pub text_state: RefCell<tui_prompts::TextState<'static>>,
}

pub(crate) struct TodoList {
    pub state: RefCell<ratatui::widgets::ListState>,
    pub tasks: persist::TaskList,
}

impl Default for TodoList {
    fn default() -> Self {
        let tasks = (1..=10)
            .map(|i| persist::Task {
                id: persist::Task::new_id(),
                title: format!("Item {}", i),
                snoozed: None,
                due: None,
                completed: None,
            })
            .collect::<Vec<_>>();
        TodoList {
            state: RefCell::new(ratatui::widgets::ListState::default()),
            tasks: persist::TaskList { tasks },
        }
    }
}

impl TodoList {
    pub(crate) fn next(&mut self) {
        let mut state = self.state.borrow_mut();
        let i = if let Some(i) = state.selected() {
            (i + 1) % self.tasks.tasks.len()
        } else {
            0
        };
        state.select(Some(i));
    }

    pub(crate) fn previous(&mut self) {
        let mut state = self.state.borrow_mut();
        let i = if let Some(i) = state.selected() {
            if i == 0 {
                self.tasks.tasks.len() - 1
            } else {
                i - 1
            }
        } else {
            0
        };
        state.select(Some(i));
    }

    pub(crate) fn unselect(&mut self) {
        self.state.borrow_mut().select(None);
    }

    pub(crate) fn toggle(&mut self) {
        if let Some(i) = self.state.borrow().selected() {
            let task = &mut self.tasks.tasks[i];
            if task.completed.is_some() {
                task.completed = None;
            } else {
                task.completed = Some(chrono::Utc::now());
            }
        }
    }
}

impl State {
    pub fn new() -> State {
        State::default()
    }

    pub fn save(&self, filename: &Path) -> Result<()> {
        let state = match &self.current_screen {
            Screen::Main(state) => &state.common_state,
            Screen::Edit(state) => &state.common_state,
        };
        let binary = persist::encode_document(&state.list.tasks)?;
        let mut file = File::create(filename)?;
        file.write_all(&binary)?;
        file.sync_all()?;
        Ok(())
    }

    pub fn load(filename: &Path) -> Result<State> {
        let mut file = File::open(filename)?;

        let mut binary = Vec::new();
        file.read_to_end(&mut binary)?;

        let tasks = persist::decode_document(&binary)?;

        Ok(State {
            current_screen: Screen::Main(MainScreenState {
                common_state: CommonState {
                    list: TodoList {
                        tasks,
                        state: RefCell::new(ratatui::widgets::ListState::default()),
                    },
                },
            }),
        })
    }
}
