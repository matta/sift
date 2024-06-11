/*!
Code for managing the displayed state of the application.

The `State` struct contains the application's state.  It is the
central data structure for the application.
*/

use std::{cell::RefCell, path::Path};

use anyhow::Result;

use crate::{main_screen::today, persist};

pub(crate) struct TodoList {
    tasks: persist::TaskList,
    selected: Option<usize>,
}

impl Default for TodoList {
    fn default() -> Self {
        let tasks: Vec<persist::Task> = (1..=10)
            .map(|i| persist::Task {
                id: persist::Task::new_id(),
                title: format!("Item {}", i),
                snoozed: None,
                due: None,
                completed: None,
            })
            .collect();
        let tasks = persist::TaskList { tasks };
        TodoList::new(tasks)
    }
}

pub(crate) fn next_week() -> chrono::NaiveDate {
    today() + chrono::TimeDelta::try_weeks(1).unwrap()
}

impl TodoList {
    fn new(tasks: persist::TaskList) -> Self {
        let selected = if tasks.tasks.is_empty() {
            None
        } else {
            Some(0)
        };
        Self { tasks, selected }
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &persist::Task> {
        self.tasks.tasks.iter()
    }

    pub(crate) fn selected(&self) -> Option<uuid::Uuid> {
        self.selected.map(|selected| self.tasks.tasks[selected].id)
    }

    pub(crate) fn index_of_id(&self, id: Option<uuid::Uuid>) -> Option<usize> {
        self.tasks.tasks.iter().enumerate().find_map(|(i, task)| {
            if Some(task.id) == id {
                Some(i)
            } else {
                None
            }
        })
    }

    fn select(&mut self, index: Option<usize>) {
        if let Some(index) = index {
            assert!(index < self.tasks.tasks.len());
        }
        self.selected = index;
    }

    fn next_index(&self) -> Option<usize> {
        let i = if let Some(i) = self.selected {
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
        let i = if let Some(i) = self.selected {
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
        if let (Some(from), Some(to)) = (self.selected, self.previous_index()) {
            self.tasks.tasks.swap(from, to);
            self.select(Some(to));
        }
    }

    pub(crate) fn move_down(&mut self) {
        if let (Some(from), Some(to)) = (self.selected, self.next_index()) {
            self.tasks.tasks.swap(from, to);
            self.select(Some(to));
        }
    }

    pub(crate) fn toggle(&mut self) {
        if let Some(i) = self.selected {
            let task = &mut self.tasks.tasks[i];
            if task.completed.is_some() {
                task.completed = None;
            } else {
                task.completed = Some(chrono::Utc::now());
            }
        }
    }

    pub(crate) fn add_task(&mut self, task: persist::Task) {
        let index = self.selected.unwrap_or(0);
        self.selected = Some(index);
        self.tasks.tasks.insert(index, task);
    }

    pub(crate) fn delete_selected(&mut self) {
        if let Some(index) = self.selected {
            // Decrement the selected item index by the number of todo
            // items up to it that will be deleted.
            let count = self
                .tasks
                .tasks
                .iter()
                .take(index)
                .filter(|task| task.completed.is_some())
                .count();
            self.selected = Some(index - count);
        }
        self.tasks.tasks.retain(|task| task.completed.is_none());
    }

    pub(crate) fn snooze(&mut self) {
        if let Some(index) = self.selected {
            let task = &mut self.tasks.tasks[index];
            task.snoozed = if task.snoozed.is_some() {
                None
            } else {
                let next_week = next_week();
                Some(next_week)
            };
        }
        // Order snoozed items after non-snoozed items.  Keep the current selection.
        //
        // Note: this is a stable sort.
        // Note: false sorts before true.
        self.tasks.tasks.sort_by_key(|task| task.snoozed.is_some());
    }

    pub(crate) fn selected_task(&self) -> Option<&persist::Task> {
        if let Some(selected) = self.selected {
            return self.tasks.tasks.get(selected);
        }
        None
    }

    pub(crate) fn set_title(&mut self, id: uuid::Uuid, title: String) {
        if let Some(task) =
            self.tasks.tasks.iter_mut().find(|task| task.id == id)
        {
            task.title = title;
        }
    }
}

#[derive(Default)]
pub(crate) struct CommonState {
    pub list: TodoList,
}

pub(crate) struct MainScreenState {
    pub common_state: CommonState,
    pub list_state: RefCell<ratatui::widgets::ListState>,
}

impl MainScreenState {
    pub(crate) fn from_common_state(common_state: CommonState) -> Self {
        Self {
            common_state,
            list_state: RefCell::new(ratatui::widgets::ListState::default()),
        }
    }
}

impl Default for MainScreenState {
    fn default() -> Self {
        MainScreenState::from_common_state(CommonState::default())
    }
}

pub(crate) struct EditScreenState {
    pub common_state: CommonState,
    pub id: uuid::Uuid,
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
                ..Default::default()
            }),
        })
    }
}
