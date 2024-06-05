//! Persistence layer

use std::collections::BTreeMap;

use automerge::Automerge;
use autosurgeon::{
    hydrate::MaybeMissing, reconcile::NoKey, Hydrate, HydrateError, Reconcile,
};
use chrono::NaiveDate;
use uuid::Uuid;

fn to_option<T>(from: MaybeMissing<T>) -> Option<T> {
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

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Task {
    /// Task identifier.
    pub id: Uuid,

    /// Title of the task.
    pub title: String,

    /// Snooze date of the task.  Optional.  The snooze date only records date
    /// information.
    pub snoozed: Option<NaiveDate>,

    /// Due date of the task.  Optional.  The due date only records date
    /// information.
    pub due: Option<NaiveDate>,

    /// Completion date and time of the task.  If `None`, the task is
    /// incomplete.
    pub completed: Option<chrono::DateTime<chrono::Utc>>,
}

impl Task {
    pub(crate) fn new_id() -> Uuid {
        let context = uuid::NoContext;
        let ts = uuid::Timestamp::now(context);
        uuid::Uuid::new_v7(ts)
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct TaskList {
    pub tasks: Vec<Task>,
}

fn make_hydrate_error(
    input: &str,
    kind: &str,
    parse_error: chrono::ParseError,
) -> HydrateError {
    HydrateError::unexpected(
        format!("error parsing {}: {}", kind, parse_error),
        input.to_string(),
    )
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct SerializableNaiveDate(NaiveDate);

impl Reconcile for SerializableNaiveDate {
    type Key<'a> = NoKey;

    fn reconcile<R: autosurgeon::Reconciler>(
        &self,
        mut reconciler: R,
    ) -> Result<(), R::Error> {
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

    fn reconcile<R: autosurgeon::Reconciler>(
        &self,
        mut reconciler: R,
    ) -> Result<(), R::Error> {
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
#[derive(
    Debug, Clone, PartialEq, autosurgeon::Reconcile, autosurgeon::Hydrate,
)]
pub(crate) struct SerializableTaskList {
    pub task_map: BTreeMap<String, SerializableTask>,
    pub task_order: Vec<String>,
}

// A SerializableTask can be created from a Task.
impl From<Task> for SerializableTask {
    fn from(value: Task) -> Self {
        Self {
            title: value.title,
            snoozed: to_maybe(value.snoozed.map(SerializableNaiveDate)),
            due_date: to_maybe(value.due.map(SerializableNaiveDate)),
            completed: to_maybe(value.completed.map(SerializableDateTime)),
        }
    }
}

// A SerializableTaskList can be created from a TaskList.
impl From<TaskList> for SerializableTaskList {
    fn from(task_list: TaskList) -> Self {
        let task_order: Vec<String> = task_list
            .tasks
            .iter()
            .map(|task| task.id.to_string())
            .collect();
        let task_map: BTreeMap<String, SerializableTask> = task_list
            .tasks
            .into_iter()
            .map(|task| (task.id.to_string(), task.into()))
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
        // TODO: this keeps the *last* of any duplicates in the task_order list.
        // We probably want to keep the first.
        let tasks: Vec<Task> = value
            .task_order
            .iter()
            .map(|id| {
                let task = value.task_map.get(id).unwrap();
                let id = Uuid::parse_str(id).unwrap();
                Task {
                    id,
                    title: task.title.clone(),
                    snoozed: to_option(task.snoozed).map(|v| v.0),
                    due: to_option(task.due_date).map(|v| v.0),
                    completed: to_option(task.completed).map(|v| v.0),
                }
            })
            .collect();
        TaskList { tasks }
    }
}

pub(crate) fn encode_document(
    tasks: &TaskList,
) -> Result<Vec<u8>, anyhow::Error> {
    let mut doc = automerge::AutoCommit::new();
    let tasks: SerializableTaskList = tasks.clone().into();
    autosurgeon::reconcile(&mut doc, tasks)?;
    Ok(doc.save())
}

pub(crate) fn decode_document(
    binary: &[u8],
) -> Result<TaskList, anyhow::Error> {
    let doc = Automerge::load(binary)?;
    let tasks: SerializableTaskList = autosurgeon::hydrate(&doc)?;
    let tasks: TaskList = tasks.into();
    Ok(tasks)
}

#[cfg(test)]
mod tests {
    use automerge_test::{assert_doc, list, map};

    use super::*;

    #[test]
    fn test() {
        let tasks = vec![
            Task {
                id: Task::new_id(),
                title: "first title".to_string(),
                snoozed: None,
                due: Some(NaiveDate::from_ymd_opt(2022, 1, 1).unwrap()),
                completed: None,
            },
            Task {
                id: Task::new_id(),
                title: "second title".to_string(),
                snoozed: Some(NaiveDate::from_ymd_opt(2022, 5, 7).unwrap()),
                due: None,
                completed: "2024-07-03T13:01:42Z"
                    .parse::<chrono::DateTime<chrono::Utc>>()
                    .ok(),
            },
        ];
        let task_list = TaskList {
            tasks: tasks.clone(),
        };

        let mut doc = automerge::AutoCommit::new();
        {
            let value: SerializableTaskList = task_list.clone().into();
            autosurgeon::reconcile(&mut doc, &value).unwrap();
        }

        assert_doc!(
            doc.document(),
            map! {
                "task_map" => {
                    map!{
                        tasks[0].id => {
                            map!{
                                "title" => {"first title"},
                                "due_date" => {"2022-01-01"},
                            }
                        },
                        tasks[1].id => {
                            map!{
                                "title" => {"second title"},
                                "snoozed" => {"2022-05-07"},
                                "completed" => {"2024-07-03T13:01:42Z"},
                            }
                        }
                    }
                },
                "task_order" => {
                    list!{
                        // FIXME: this is slightly convoluted. It would be nice
                        // if the .as_str() was unecessary.
                        // https://github.com/automerge/automerge/issues/926
                        {tasks[0].id.to_string().as_str()},
                        {tasks[1].id.to_string().as_str()},
                    }
                },
            }
        );

        let todo_list2: SerializableTaskList =
            autosurgeon::hydrate(&doc).unwrap();
        let todo_list2: TaskList = todo_list2.into();
        assert_eq!(task_list, todo_list2);
    }
}
