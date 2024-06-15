/*!
Code for managing the displayed state of the application.

The `State` struct contains the application's state.  It is the
central data structure for the application.
*/

use std::path::Path;

use anyhow::Result;
use chrono::Datelike;
use itertools::Itertools;

use crate::persist::{MemoryStore, Store, Task, TaskId};
use crate::screen::{self, Screen};

fn today() -> chrono::NaiveDate {
    let now = chrono::Local::now();
    chrono::NaiveDate::from_ymd_opt(now.year(), now.month(), now.day()).unwrap()
}

fn next_week() -> chrono::NaiveDate {
    today() + chrono::TimeDelta::try_weeks(1).unwrap()
}

#[derive(Default)]
pub(crate) struct CommonState {
    pub store: MemoryStore,
    pub selected: Option<TaskId>,
}

impl CommonState {
    pub(crate) fn new(store: MemoryStore) -> Self {
        let mut state = CommonState {
            store,
            selected: None,
        };
        state.selected = state.first_id();
        state
    }

    pub(crate) fn index_of_id(&mut self, id: Option<TaskId>) -> Option<usize> {
        self.list_tasks_for_display()
            .into_iter()
            .enumerate()
            .find_map(
                |(i, task)| {
                    if Some(task.id()) == id {
                        Some(i)
                    } else {
                        None
                    }
                },
            )
    }

    pub fn list_tasks_for_display(&mut self) -> Vec<Task> {
        let today = today();
        let mut tasks = self.store.list_tasks().expect("XXX: handle error");
        tasks.retain(|task| {
            let snoozed = matches!(task.snoozed(), Some(date) if date > today);
            !snoozed
        });
        tasks
    }

    pub fn toggle(&mut self) {
        if let Some(id) = self.selected {
            let mut task = self
                .store
                .get_task(&id)
                .expect("FIXME: propagate errors; selected task must be in the store");
            let completed = if task.completed().is_some() {
                None
            } else {
                Some(chrono::Utc::now())
            };
            task.set_completed(completed);
            self.store.put_task(&task).expect("FIXME: propagate errors");
        }
    }

    pub(crate) fn snooze(&mut self) {
        if let Some(id) = self.selected {
            let mut task = self
                .store
                .get_task(&id)
                .expect("FIXME: propagate errors; selected task must be in the store");
            let snoozed = if task.snoozed().is_some() {
                None
            } else {
                let next_week = next_week();
                Some(next_week)
            };
            task.set_snoozed(snoozed);
            self.store.put_task(&task).expect("FIXME: propagate errors");
        }
        // Order snoozed items after non-snoozed items.  Keep the current
        // selection.
        //
        // Note: this is a stable sort.
        // Note: false sorts before true.
        // self.tasks
        //     .tasks
        //     .sort_by_key(|task| task.snoozed().is_some());
    }

    fn id_of_index(&mut self, index: usize) -> Option<TaskId> {
        self.list_tasks_for_display()
            .into_iter()
            .enumerate()
            .find_map(|(i, task)| if i == index { Some(task.id()) } else { None })
    }

    fn first_id(&mut self) -> Option<TaskId> {
        self.list_tasks_for_display()
            .into_iter()
            .map(|task| task.id())
            .next()
    }

    fn next_id(&mut self) -> Option<TaskId> {
        if let Some(selected) = self.selected {
            for (prev, next) in self
                .list_tasks_for_display()
                .into_iter()
                .map(|task| task.id())
                .tuple_windows()
            {
                if prev == selected {
                    return Some(next);
                }
            }
        }
        self.first_id()
    }

    fn previous_id(&mut self) -> Option<TaskId> {
        let mut last = None;
        for (prev, next) in self
            .list_tasks_for_display()
            .into_iter()
            .map(|task| task.id())
            .tuple_windows()
        {
            if Some(next) == self.selected {
                return Some(prev);
            }
            last = Some(next);
        }
        last
    }

    pub(crate) fn next(&mut self) {
        self.selected = self.next_id();
    }

    pub(crate) fn previous(&mut self) {
        self.selected = self.previous_id();
    }

    pub(crate) fn move_up(&mut self) {
        if let (Some(id), Some(index)) = (self.selected, self.index_of_id(self.selected)) {
            let previous = index.checked_sub(1).and_then(|i| self.id_of_index(i));
            self.store
                .move_task(previous.as_ref(), &id)
                .expect("FIXME: handle this error");
        }
    }

    pub(crate) fn move_down(&mut self) {
        let len = self.list_tasks_for_display().len();
        if let (Some(selected), Some(index)) = (self.selected, self.index_of_id(self.selected)) {
            let next_index = if index == len - 1 {
                if index == 0 {
                    return;
                }
                0
            } else {
                index + 1
            };
            let next = self.id_of_index(next_index);
            self.store
                .move_task(next.as_ref(), &selected)
                .expect("FIXME: propagate errors from here");
        }
    }

    pub(crate) fn delete_selected(&mut self) {
        let mut deletions = Vec::new();
        let mut new_selected = None;
        let mut saw_selected = false;

        for task in self.list_tasks_for_display() {
            if task.completed().is_some() {
                deletions.push(task.id());
            } else if !saw_selected {
                new_selected = Some(task.id());
            }
            if let Some(selected) = self.selected {
                if task.id() == selected {
                    saw_selected = true;
                }
            }
        }
        self.selected = new_selected;

        for id in &deletions {
            self.store
                .delete_task(id)
                .expect("FIXME: handle error here");
        }
    }
}

pub(crate) struct State {
    // FIXME: make non-public
    pub common_state: CommonState,
    pub current_screen: Option<Box<dyn Screen>>,
}

impl Default for State {
    fn default() -> Self {
        let current_screen = Some(Box::<dyn Screen>::from(
            Box::new(screen::main::State::new()),
        ));
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

    pub fn save(&self, path: &Path) -> Result<()> {
        self.common_state.store.save(path)
    }

    pub fn load(path: &Path) -> Result<State> {
        let store = MemoryStore::load(path)?;
        let common_state = CommonState::new(store);
        let state = State {
            common_state,
            ..Default::default()
        };
        Ok(state)
    }
}
