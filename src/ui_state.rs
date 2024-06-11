/*!
Code for managing the displayed state of the application.

The `State` struct contains the application's state.  It is the
central data structure for the application.
*/

use std::{cell::RefCell, path::Path};

use anyhow::Result;

use crate::persist;

pub(crate) struct TodoList {
    pub list_state: RefCell<ratatui::widgets::ListState>,
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
            list_state: RefCell::new(ratatui::widgets::ListState::default()),
            tasks: persist::TaskList { tasks },
        }
    }
}

impl TodoList {
    fn current_index(&self) -> Option<usize> {
        self.list_state.borrow_mut().selected()
    }

    fn select(&mut self, index: Option<usize>) {
        if let Some(index) = index {
            assert!(index < self.tasks.tasks.len());
        }
        self.list_state.borrow_mut().select(index);
    }

    fn next_index(&self) -> Option<usize> {
        let i = if let Some(i) = self.current_index() {
            i.saturating_add(1) % self.tasks.tasks.len()
        } else {
            0
        };
        if i < self.tasks.tasks.len() {
            Some(i)
        } else {
            None
        }
    }

    fn previous_index(&self) -> Option<usize> {
        let i = if let Some(i) = self.current_index() {
            if i == 0 {
                self.tasks.tasks.len().saturating_sub(1)
            } else {
                i - 1
            }
        } else {
            0
        };
        if i < self.tasks.tasks.len() {
            Some(i)
        } else {
            None
        }
    }

    pub(crate) fn next(&mut self) {
        self.select(self.next_index());
    }

    pub(crate) fn previous(&mut self) {
        self.select(self.previous_index());
    }

    pub(crate) fn move_up(&mut self) {
        if let (Some(from), Some(to)) =
            (self.current_index(), self.previous_index())
        {
            self.tasks.tasks.swap(from, to);
            self.select(Some(to));
        }
    }

    pub(crate) fn move_down(&mut self) {
        if let (Some(from), Some(to)) =
            (self.current_index(), self.next_index())
        {
            self.tasks.tasks.swap(from, to);
            self.select(Some(to));
        }
    }

    pub(crate) fn toggle(&mut self) {
        if let Some(i) = self.list_state.borrow().selected() {
            let task = &mut self.tasks.tasks[i];
            if task.completed.is_some() {
                task.completed = None;
            } else {
                task.completed = Some(chrono::Utc::now());
            }
        }
    }

    fn new(tasks: persist::TaskList) -> Self {
        let selected = if tasks.tasks.is_empty() {
            None
        } else {
            Some(0)
        };
        let list_state =
            ratatui::widgets::ListState::default().with_selected(selected);
        Self {
            tasks,
            list_state: RefCell::new(list_state),
        }
    }
}

#[derive(Default)]
pub(crate) struct CommonState {
    pub list: TodoList,
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
pub(crate) struct State {
    pub current_screen: Screen,
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
        persist::save_tasks(filename, &state.list.tasks)?;
        Ok(())
    }

    pub fn load(filename: &Path) -> Result<State> {
        let tasks = persist::load_tasks(filename)?;

        Ok(State {
            current_screen: Screen::Main(MainScreenState {
                common_state: CommonState {
                    list: TodoList::new(tasks),
                },
            }),
        })
    }
}
