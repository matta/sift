use std::collections::{BTreeMap, HashSet};

use autosurgeon::reconcile::NoKey;
use autosurgeon::{Hydrate, HydrateError, MaybeMissing, Reconcile};
use chrono::NaiveDate;
use uuid::Uuid;

use super::Task;
use crate::persist::document::TaskList;

pub fn to_option<T>(from: MaybeMissing<T>) -> Option<T> {
    match from {
        MaybeMissing::Missing => None,
        MaybeMissing::Present(v) => Some(v),
    }
}

fn to_maybe<T>(from: Option<T>) -> MaybeMissing<T> {
    match from {
        None => MaybeMissing::Missing,
        Some(v) => MaybeMissing::Present(v),
    }
}

fn make_hydrate_error(input: &str, kind: &str, parse_error: chrono::ParseError) -> HydrateError {
    HydrateError::unexpected(
        format!("error parsing {}: {}", kind, parse_error),
        input.to_string(),
    )
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct SerializableNaiveDate(NaiveDate);

impl Reconcile for SerializableNaiveDate {
    type Key<'a> = NoKey;

    fn reconcile<R: autosurgeon::Reconciler>(&self, mut reconciler: R) -> Result<(), R::Error> {
        reconciler.str(self.0.format("%F").to_string())
    }
}

impl Hydrate for SerializableNaiveDate {
    fn hydrate_string(s: &'_ str) -> Result<Self, HydrateError> {
        match s.parse::<NaiveDate>() {
            Ok(d) => Ok(SerializableNaiveDate(d)),
            Err(e) => Err(make_hydrate_error(s, "naive date", e)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct SerializableDateTime(chrono::DateTime<chrono::Utc>);

impl Reconcile for SerializableDateTime {
    type Key<'a> = NoKey;

    fn reconcile<R: autosurgeon::Reconciler>(&self, mut reconciler: R) -> Result<(), R::Error> {
        reconciler.str(self.0.format("%FT%TZ").to_string())
    }
}

impl Hydrate for SerializableDateTime {
    fn hydrate_string(s: &'_ str) -> Result<Self, HydrateError> {
        match s.parse::<chrono::DateTime<chrono::Utc>>() {
            Ok(d) => Ok(SerializableDateTime(d)),
            Err(e) => Err(make_hydrate_error(s, "date time", e)),
        }
    }
}

// SerializableTask is a Task that can be stored and retrieved from an
// Automerge document.
#[derive(Debug, Clone, PartialEq, Reconcile, Hydrate)]
pub(crate) struct SerializableTask {
    pub title: String,
    pub snoozed: autosurgeon::hydrate::MaybeMissing<SerializableNaiveDate>,
    pub due_date: autosurgeon::hydrate::MaybeMissing<SerializableNaiveDate>,
    pub completed: autosurgeon::hydrate::MaybeMissing<SerializableDateTime>,
}

// SerializableTaskList is a TaskList that can be stored and retrieved from
// an Automerge document.
#[derive(Debug, Clone, PartialEq, autosurgeon::Reconcile, autosurgeon::Hydrate)]
pub(crate) struct SerializableTaskList {
    pub task_map: BTreeMap<String, SerializableTask>,
    pub task_order: Vec<String>,
}

// A SerializableTask can be created from a Task.
impl From<Task> for SerializableTask {
    fn from(value: Task) -> Self {
        Self {
            title: value.title().into(),
            snoozed: to_maybe(value.snoozed().map(SerializableNaiveDate)),
            due_date: to_maybe(value.due().map(SerializableNaiveDate)),
            completed: to_maybe(value.completed().map(SerializableDateTime)),
        }
    }
}

// A SerializableTaskList can be created from a TaskList.
impl From<TaskList> for SerializableTaskList {
    fn from(task_list: TaskList) -> Self {
        let task_order: Vec<String> = task_list
            .tasks
            .iter()
            .map(|task| task.id().to_string())
            .collect();
        let task_map: BTreeMap<String, SerializableTask> = task_list
            .tasks
            .into_iter()
            .map(|task| (task.id().to_string(), task.into()))
            .collect();
        Self {
            task_map,
            task_order,
        }
    }
}

// A TaskList can be created from a SerializableTaskList.
impl From<SerializableTaskList> for TaskList {
    fn from(value: SerializableTaskList) -> Self {
        let mut seen = HashSet::new();
        let tasks: Vec<Task> = value
            .task_order
            .iter()
            .filter_map(|id| {
                let task = value.task_map.get(id).unwrap();
                let id = Uuid::parse_str(id).unwrap();
                if !seen.insert(id) {
                    // Due to CRDT merges an item may appear in multiple places
                    // in the automerge doc.  Ignore all but
                    // the first.
                    return None;
                }
                Some(Task::new(
                    id.into(),
                    task.title.clone(),
                    to_option(task.snoozed).map(|v| v.0),
                    to_option(task.due_date).map(|v| v.0),
                    to_option(task.completed).map(|v| v.0),
                ))
            })
            .collect();
        TaskList { tasks }
    }
}
