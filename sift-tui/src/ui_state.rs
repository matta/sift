/*!
Code for managing the displayed state of the application.

The `State` struct contains the application's state.  It is the
central data structure for the application.
*/

use std::path::Path;

use anyhow::Result;
use chrono::Datelike;
use itertools::Itertools;
use sift_persist::{MemoryStore, Store, Task, TaskId};

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
            self.store
                .with_transaction(|txn| {
                    let mut task = txn.get_task(&id)?;
                    let completed = if task.completed().is_some() {
                        None
                    } else {
                        Some(chrono::Utc::now())
                    };
                    task.set_completed(completed);
                    txn.put_task(&task)?;
                    Ok(())
                })
                .expect("FIXME: propagate errors");
        }
    }

    pub(crate) fn snooze(&mut self) {
        if let Some(id) = self.selected {
            self.store
                .with_transaction(|txn| {
                    let mut task = txn.get_task(&id)?;
                    let snoozed = if task.snoozed().is_some() {
                        None
                    } else {
                        let next_week = next_week();
                        Some(next_week)
                    };
                    task.set_snoozed(snoozed);
                    txn.put_task(&task)
                })
                .expect("FIXME: propagate errors");
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
        if let Some(selected) = self.selected {
            let ids = self.task_ids_for_move();
            for (prev_prev_id, _, id) in ids.iter().circular_tuple_windows() {
                if *id == Some(selected) {
                    self.store
                        .with_transaction(|txn| txn.move_task(prev_prev_id.as_ref(), &selected))
                        .expect("FIXME: handle this error");
                    break;
                }
            }
        }
    }

    // Return the list of displayed task IDs in a format useful for moving. The
    // first id is always None, and subsequent ones are valid IDs in the usual
    // order. This is useful for determining the previous task ID, which is None
    // for the first real task.
    //
    // Not sure this is the best API. Consider returning Vec<TaskId> and making
    // callers deal with wrapping, etc.
    fn task_ids_for_move(&mut self) -> Vec<Option<TaskId>> {
        std::iter::once(None)
            .chain(
                self.list_tasks_for_display()
                    .iter()
                    .map(|task| Some(task.id())),
            )
            .collect()
    }

    pub(crate) fn move_down(&mut self) {
        if let Some(selected) = self.selected {
            let ids = self.task_ids_for_move();
            for (id, successor_id) in ids.iter().circular_tuple_windows() {
                if *id == Some(selected) {
                    self.store
                        .with_transaction(|txn| txn.move_task(successor_id.as_ref(), &selected))
                        .expect("FIXME: handle this error");
                    break;
                }
            }
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

        self.store
            .with_transaction(|txn| {
                for id in &deletions {
                    txn.delete_task(id).expect("FIXME: handle error here");
                }
                Ok(())
            })
            .expect("TODO: handle errors here");
    }

    pub(crate) fn undo(&mut self) {
        let _ignored = self.store.undo();
    }

    pub(crate) fn redo(&mut self) {
        let _ignored = self.store.redo();
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
