use std::collections::BTreeMap;

use automerge::Automerge;
use chrono::NaiveDate;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Task {
    pub id: Uuid,
    pub title: String,
    pub snoozed: Option<NaiveDate>,
    pub due_date: Option<NaiveDate>,
    pub completed: bool,
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
    pub tasks: BTreeMap<Uuid, Task>,
}

// SerializableTask is a Task that can be stored and retrieved from an
// Automerge document.
#[derive(Debug, Clone, PartialEq, autosurgeon::Reconcile, autosurgeon::Hydrate)]
pub(crate) struct SerializableTask {
    pub title: String,
    #[autosurgeon(with = "option_naive_date")]
    pub snoozed: Option<NaiveDate>,
    #[autosurgeon(with = "option_naive_date")]
    pub due_date: Option<NaiveDate>,
    pub completed: bool,
}

// SerializableTaskList is a TaskList that can be stored and retrieved from
// an Automerge document.
#[derive(Debug, Clone, PartialEq, autosurgeon::Reconcile, autosurgeon::Hydrate)]
pub(crate) struct SerializableTaskList {
    pub tasks: BTreeMap<String, SerializableTask>,
}

// A SerializableTask can be created from a Task.
impl From<Task> for SerializableTask {
    fn from(value: Task) -> Self {
        Self {
            title: value.title,
            snoozed: value.snoozed,
            due_date: value.due_date,
            completed: value.completed,
        }
    }
}

// A SerializableTaskList can be created from a TaskList.
impl From<TaskList> for SerializableTaskList {
    fn from(task_list: TaskList) -> Self {
        let mut converted_tasks: BTreeMap<String, SerializableTask> = BTreeMap::new();
        for (id, task) in task_list.tasks.into_iter() {
            assert_eq!(id, task.id);
            converted_tasks.insert(id.to_string(), task.into());
        }
        Self {
            tasks: converted_tasks,
        }
    }
}

// A TaskList can be created from a SerializableTaskList.
impl From<SerializableTaskList> for TaskList {
    fn from(value: SerializableTaskList) -> Self {
        let mut result = TaskList::default();
        for (id, serializable_task) in value.tasks.into_iter() {
            let task: Task = Task {
                id: Uuid::parse_str(&id).unwrap(),
                title: serializable_task.title,
                snoozed: serializable_task.snoozed,
                due_date: serializable_task.due_date,
                completed: serializable_task.completed,
            };
            result.tasks.insert(task.id, task);
        }
        result
    }
}

/// Reconcile an `Option<NaiveDate>` value with an optional string in
/// an automerge document.
///a
/// This helper module is used with the #[autosurgeon(with = "option_naive_date")]
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

pub(crate) fn encode_document(tasks: &TaskList) -> Result<Vec<u8>, anyhow::Error> {
    let mut doc = automerge::AutoCommit::new();
    let tasks: SerializableTaskList = tasks.clone().into();
    autosurgeon::reconcile(&mut doc, tasks)?;
    Ok(doc.save())
}

pub(crate) fn decode_document(binary: Vec<u8>) -> Result<TaskList, anyhow::Error> {
    let doc = Automerge::load(&binary)?;
    let tasks: SerializableTaskList = autosurgeon::hydrate(&doc)?;
    let tasks: TaskList = tasks.into();
    Ok(tasks)
}

#[cfg(test)]
mod tests {
    use automerge::ScalarValue;
    use automerge_test::{assert_doc, map};

    use super::*;

    #[test]
    fn test() {
        let tasks = vec![
            Task {
                id: Task::new_id(),
                title: "first title".to_string(),
                snoozed: None,
                due_date: Some(NaiveDate::from_ymd_opt(2022, 1, 1).unwrap()),
                completed: false,
            },
            Task {
                id: Task::new_id(),
                title: "second title".to_string(),
                snoozed: Some(NaiveDate::from_ymd_opt(2022, 5, 7).unwrap()),
                due_date: None,
                completed: false,
            },
        ];
        let todo_list = TaskList {
            tasks: tasks.iter().map(|task| (task.id, task.clone())).collect(),
        };

        let mut doc = automerge::AutoCommit::new();
        {
            let value: SerializableTaskList = todo_list.clone().into();
            autosurgeon::reconcile(&mut doc, &value).unwrap();
        }

        assert_doc!(
            doc.document(),
            map! {
                "tasks" => {
                    map!{
                        tasks[0].id => {
                            map!{
                                "title" => {"first title"},
                                "snoozed" => {ScalarValue::Null},
                                "due_date" => {"2022-01-01"},
                                "completed" => {false},
                            }
                        },
                        tasks[1].id => {
                            map!{
                                "title" => {"second title"},
                                "snoozed" => {"2022-05-07"},
                                "due_date" => {ScalarValue::Null},
                                "completed" => {false},
                            }
                        }
                    }
                },
            }
        );

        let todo_list2: SerializableTaskList = autosurgeon::hydrate(&doc).unwrap();
        let todo_list2: TaskList = todo_list2.into();
        assert_eq!(todo_list, todo_list2);
    }
}
