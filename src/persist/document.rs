use crate::persist::serialization::SerializableTaskList;
use automerge::Automerge;
use chrono::NaiveDate;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use uuid::Uuid;

use super::container::{
    read_chunk, read_header, write_chunk, write_header, Chunk,
};

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

fn encode_document(tasks: &TaskList) -> Result<Vec<u8>, anyhow::Error> {
    let mut doc = automerge::AutoCommit::new();
    let tasks: SerializableTaskList = tasks.clone().into();
    autosurgeon::reconcile(&mut doc, tasks)?;
    Ok(doc.save())
}

fn decode_document(binary: &[u8]) -> Result<TaskList, anyhow::Error> {
    let doc = Automerge::load(binary)?;
    let tasks: SerializableTaskList = autosurgeon::hydrate(&doc)?;
    let tasks: TaskList = tasks.into();
    Ok(tasks)
}

const AUTOMERGE_CHUNK: [u8; 4] = [b'A', b'M', b'R', b'G'];
const END_CHUNK: [u8; 4] = [b'S', b'E', b'N', b'D'];

// TODO: use a custom error type. See
// https://www.reddit.com/r/rust/comments/wtu5te/how_should_i_propagate_my_errors_to_include_a/
fn write_document<W: Write>(
    tasks: &TaskList,
    writer: &mut W,
) -> anyhow::Result<(), anyhow::Error> {
    write_header(writer)?;
    let chunk = Chunk::new(AUTOMERGE_CHUNK, encode_document(tasks)?);
    write_chunk(&chunk, writer)?;
    let chunk = Chunk::new(END_CHUNK, vec![]);
    write_chunk(&chunk, writer)?;
    Ok(())
}

// TODO: use a custom error type. See
// https://www.reddit.com/r/rust/comments/wtu5te/how_should_i_propagate_my_errors_to_include_a/
fn expect_type(
    chunk: &Chunk,
    expected_type: [u8; 4],
) -> Result<(), std::io::Error> {
    if chunk.chunk_type == expected_type {
        Ok(())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "unexpected chunk type",
        ))
    }
}

// TODO: use a custom error type. See
// https://www.reddit.com/r/rust/comments/wtu5te/how_should_i_propagate_my_errors_to_include_a/
fn read_document<R: Read>(
    reader: &mut R,
) -> anyhow::Result<TaskList, anyhow::Error> {
    read_header(reader)?;
    let automerge_chunk = read_chunk(reader)?;
    expect_type(&automerge_chunk, AUTOMERGE_CHUNK)?;

    let tasks = decode_document(&automerge_chunk.data)?;

    let end_chunk = read_chunk(reader)?;
    expect_type(&end_chunk, END_CHUNK)?;

    Ok(tasks)
}

// TODO: use a custom error type. See
// https://www.reddit.com/r/rust/comments/wtu5te/how_should_i_propagate_my_errors_to_include_a/
pub fn save_tasks(
    filename: &Path,
    tasks: &TaskList,
) -> anyhow::Result<(), anyhow::Error> {
    let mut file = File::create(filename)?;
    write_document(tasks, &mut file)?;
    file.sync_all()?;
    Ok(())
}

// TODO: use a custom error type. See
// https://www.reddit.com/r/rust/comments/wtu5te/how_should_i_propagate_my_errors_to_include_a/
pub fn load_tasks(filename: &Path) -> anyhow::Result<TaskList, anyhow::Error> {
    let mut file = File::open(filename)?;
    read_document(&mut file)
}

#[cfg(test)]
mod tests {
    use automerge_test::{assert_doc, list, map};

    use crate::persist::document::{Task, TaskList};
    use crate::persist::*;

    #[test]
    fn test() {
        let tasks = vec![
            Task {
                id: Task::new_id(),
                title: "first title".to_string(),
                snoozed: None,
                due: Some(chrono::NaiveDate::from_ymd_opt(2022, 1, 1).unwrap()),
                completed: None,
            },
            Task {
                id: Task::new_id(),
                title: "second title".to_string(),
                snoozed: Some(
                    chrono::NaiveDate::from_ymd_opt(2022, 5, 7).unwrap(),
                ),
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
            let value: serialization::SerializableTaskList =
                task_list.clone().into();
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

        let todo_list2: serialization::SerializableTaskList =
            autosurgeon::hydrate(&doc).unwrap();
        let todo_list2: TaskList = todo_list2.into();
        assert_eq!(task_list, todo_list2);
    }
}
