/*!
Code for managing the displayed state of the application.

The `State` struct contains the application's state.  It is the
central data structure for the application.
*/

use crate::{persist, screen};
use anyhow::Result;
use chrono::Datelike;
use std::path::Path;

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

fn today() -> chrono::NaiveDate {
    let now = chrono::Local::now();
    chrono::NaiveDate::from_ymd_opt(now.year(), now.month(), now.day()).unwrap()
}

fn next_week() -> chrono::NaiveDate {
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
        let today = today();
        self.tasks.tasks.iter().filter(move |task| {
            let snoozed = matches!(task.snoozed, Some(date) if date > today);
            !snoozed
        })
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
        let visible = self.iter().count();
        if visible == 0 {
            return None;
        }
        Some(if let Some(i) = self.selected {
            i.wrapping_add(1) % visible
        } else {
            0
        })
    }

    fn previous_index(&self) -> Option<usize> {
        let visible = self.iter().count();
        if visible == 0 {
            return None;
        }
        Some(match self.selected {
            Some(i) => {
                if i == 0 {
                    visible - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        })
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

pub(crate) type Screen = dyn screen::Screen<Context = CommonState>;

pub(crate) struct State {
    // FIXME: make non-public
    pub common_state: CommonState,
    pub current_screen: Option<Box<Screen>>,
}

impl Default for State {
    fn default() -> Self {
        let current_screen =
            Some(Box::<Screen>::from(Box::new(screen::main::State::new())));
        State {
            common_state: CommonState::default(),
            current_screen,
        }
    }
}

impl State {
    pub fn new() -> State {
        State::default()
    }

    pub fn save(&self, filename: &Path) -> Result<()> {
        persist::save_tasks(filename, &self.common_state.list.tasks)?;
        Ok(())
    }

    pub fn load(filename: &Path) -> Result<State> {
        let tasks = persist::load_tasks(filename)?;
        let common_state = CommonState {
            list: TodoList::new(tasks),
        };
        let state = State {
            common_state,
            ..Default::default()
        };
        Ok(state)
    }
}
