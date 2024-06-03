//! Persistence layer

use std::collections::BTreeMap;

use automerge::Automerge;
use chrono::NaiveDate;
use uuid::Uuid;

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

// SerializableTask is a Task that can be stored and retrieved from an
// Automerge document.
#[derive(
    Debug, Clone, PartialEq, autosurgeon::Reconcile, autosurgeon::Hydrate,
)]
pub(crate) struct SerializableTask {
    pub title: String,
    #[autosurgeon(with = "option_naive_date")]
    pub snoozed: Option<NaiveDate>,
    #[autosurgeon(with = "option_naive_date")]
    pub due_date: Option<NaiveDate>,
    #[autosurgeon(with = "option_date_time")]
    pub completed: Option<chrono::DateTime<chrono::Utc>>,
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
            snoozed: value.snoozed,
            due_date: value.due,
            completed: value.completed,
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
                    snoozed: task.snoozed,
                    due: task.due_date,
                    completed: task.completed,
                }
            })
            .collect();
        TaskList { tasks }
    }
}

/// Reconcile an `Option<NaiveDate>` value with an optional string in
/// an automerge document.
///a
/// This helper module is used with the
// `#[autosurgeon(with = "option_naive_date")]`
/// syntax.
mod option_date_time {
    use autosurgeon::{Hydrate, HydrateError, Prop, ReadDoc, Reconciler};

    pub(super) fn hydrate<D: ReadDoc>(
        doc: &D,
        obj: &automerge::ObjId,
        prop: Prop<'_>,
    ) -> Result<Option<chrono::DateTime<chrono::Utc>>, HydrateError> {
        type OptionString = Option<String>;
        let inner = OptionString::hydrate(doc, obj, prop)?;
        match inner {
            None => Ok(None),
            Some(s) => match s.parse::<chrono::DateTime<chrono::Utc>>() {
                Ok(d) => Ok(Some(d)),
                Err(_) => Err(HydrateError::unexpected(
                    "a valid date in YYYY-MM-DD format",
                    s,
                )),
            },
        }
    }

    // Given an `Option<NaiveDate>` value, write either a none value or
    // a string in the format YYYY-MM-DD.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub(super) fn reconcile<R: Reconciler>(
        date: &Option<chrono::DateTime<chrono::Utc>>,
        mut reconciler: R,
    ) -> Result<(), R::Error> {
        match date {
            None => reconciler.none(),
            Some(d) => reconciler.str(d.format("%FT%TZ").to_string()),
        }
    }
}

/// Reconcile an `Option<NaiveDate>` value with an optional string in
/// an automerge document.
///a
/// This helper module is used with the
// `#[autosurgeon(with = "option_naive_date")]`
/// syntax.
mod option_naive_date {
    use autosurgeon::{Hydrate, HydrateError, Prop, ReadDoc, Reconciler};
    use chrono::NaiveDate;

    /// Create a new `Option<NaiveDate>` value from a, possibly missing,
    /// string in an automerge document.
    ///
    /// May return an error if the string is not in the format YYYY-MM-DD
    /// or not a valid date.
    pub(super) fn hydrate<D: ReadDoc>(
        doc: &D,
        obj: &automerge::ObjId,
        prop: Prop<'_>,
    ) -> Result<Option<NaiveDate>, HydrateError> {
        type OptionString = Option<String>;
        let inner = OptionString::hydrate(doc, obj, prop)?;
        match inner {
            None => Ok(None),
            Some(s) => match s.parse::<NaiveDate>() {
                Ok(d) => Ok(Some(d)),
                Err(_) => Err(HydrateError::unexpected(
                    "a valid date in YYYY-MM-DD format",
                    s,
                )),
            },
        }
    }

    // Given an `Option<NaiveDate>` value, write either a none value or
    // a string in the format YYYY-MM-DD.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub(super) fn reconcile<R: Reconciler>(
        date: &Option<NaiveDate>,
        mut reconciler: R,
    ) -> Result<(), R::Error> {
        match date {
            None => reconciler.none(),
            Some(d) => reconciler.str(d.format("%F").to_string()),
        }
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
    use automerge::ScalarValue;
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
                                "snoozed" => {ScalarValue::Null},
                                "due_date" => {"2022-01-01"},
                                "completed" => {ScalarValue::Null},
                            }
                        },
                        tasks[1].id => {
                            map!{
                                "title" => {"second title"},
                                "snoozed" => {"2022-05-07"},
                                "due_date" => {ScalarValue::Null},
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
