use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use automerge::Automerge;

use super::container::{self, read_chunk, read_header, write_chunk, write_header, Chunk};
use super::Task;
use crate::serialization::SerializableTaskList;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Cannot create file `{1}`")]
    CreateFile(#[source] std::io::Error, PathBuf),
    #[error("Cannot open file `{1}`")]
    OpenFile(#[source] std::io::Error, PathBuf),
    #[error("Cannot write to file")]
    Write(#[source] std::io::Error),
    #[error("Cannot write container item to file")]
    ContainerWrite(#[source] container::Error),
    #[error("Cannot read container item to file")]
    ContainerRead(#[source] container::Error),
    #[error("Cannot load automerge document")]
    AutomergeLoad(#[source] automerge::AutomergeError),
    #[error("Cannot reconcile program state as an automerge document")]
    Reconcile(#[source] autosurgeon::ReconcileError),
    #[error("Cannot hydrate from automerge document")]
    Hydrate(#[source] autosurgeon::HydrateError),
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct TaskList {
    pub tasks: Vec<Task>,
}

fn encode_document(tasks: &TaskList) -> Result<Vec<u8>, Error> {
    let mut doc = automerge::AutoCommit::new();
    let tasks: SerializableTaskList = tasks.clone().into();
    autosurgeon::reconcile(&mut doc, tasks).map_err(Error::Reconcile)?;
    Ok(doc.save())
}

fn decode_document(binary: &[u8]) -> Result<TaskList, Error> {
    let doc = Automerge::load(binary).map_err(Error::AutomergeLoad)?;
    let tasks: SerializableTaskList = autosurgeon::hydrate(&doc).map_err(Error::Hydrate)?;
    let tasks: TaskList = tasks.into();
    Ok(tasks)
}

const AUTOMERGE_CHUNK: [u8; 4] = [b'A', b'M', b'R', b'G'];
const END_CHUNK: [u8; 4] = [b'S', b'E', b'N', b'D'];

fn write_document<W: Write>(tasks: &TaskList, writer: &mut W) -> Result<(), Error> {
    write_header(writer).map_err(Error::ContainerWrite)?;
    let chunk = Chunk::new(AUTOMERGE_CHUNK, encode_document(tasks)?);
    write_chunk(&chunk, writer).map_err(Error::ContainerWrite)?;
    let chunk = Chunk::new(END_CHUNK, vec![]);
    write_chunk(&chunk, writer).map_err(Error::ContainerWrite)?;
    Ok(())
}

fn read_document<R: Read>(reader: &mut R) -> Result<TaskList, Error> {
    read_header(reader).map_err(Error::ContainerRead)?;
    let automerge_chunk = read_chunk(reader).map_err(Error::ContainerRead)?;
    automerge_chunk
        .expect_type(AUTOMERGE_CHUNK)
        .map_err(Error::ContainerRead)?;

    let tasks = decode_document(&automerge_chunk.data)?;

    let end_chunk = read_chunk(reader).map_err(Error::ContainerRead)?;
    end_chunk
        .expect_type(END_CHUNK)
        .map_err(Error::ContainerRead)?;

    Ok(tasks)
}

pub fn save_tasks(filename: &Path, tasks: &TaskList) -> Result<(), Error> {
    let mut file = File::create(filename).map_err(|e| Error::CreateFile(e, filename.to_owned()))?;
    write_document(tasks, &mut file)?;
    file.sync_all().map_err(Error::Write)?;
    Ok(())
}

pub fn load_tasks(filename: &Path) -> Result<TaskList, Error> {
    let mut file = File::open(filename).map_err(|e| Error::OpenFile(e, filename.to_owned()))?;
    // TODO: the file name is not reported for errors returned by read_document.
    // It would probably be better for the container module to return only
    // std::io::Error, and wrap all std::io::Error in more general Read and
    // Write errors in this module.
    read_document(&mut file)
}

#[cfg(test)]
mod tests {
    use automerge_test::{assert_doc, list, map};

    use crate::document::{Task, TaskList};
    use crate::serialization::SerializableTaskList;

    #[test]
    fn test() {
        let new_date = |year, month, day| -> chrono::NaiveDate {
            chrono::NaiveDate::from_ymd_opt(year, month, day).expect("date must parse")
        };

        let tasks = vec![
            Task::new(
                Task::new_id(),
                "first title".to_string(),
                None,
                Some(new_date(2022, 1, 1)),
                None,
            ),
            Task::new(
                Task::new_id(),
                "second title".to_string(),
                Some(new_date(2022, 5, 7)),
                None,
                "2024-07-03T13:01:42Z"
                    .parse::<chrono::DateTime<chrono::Utc>>()
                    .ok(),
            ),
        ];
        let task_list = TaskList {
            tasks: tasks.clone(),
        };

        let mut doc = automerge::AutoCommit::new();
        {
            let value: SerializableTaskList = task_list.clone().into();
            autosurgeon::reconcile(&mut doc, &value).expect("reconcile must succeed");
        }

        assert_doc!(
            doc.document(),
            map! {
                "task_map" => {
                    map!{
                        tasks[0].id() => {
                            map!{
                                "title" => {"first title"},
                                "due_date" => {"2022-01-01"},
                            }
                        },
                        tasks[1].id() => {
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
                        {tasks[0].id().to_string().as_str()},
                        {tasks[1].id().to_string().as_str()},
                    }
                },
            }
        );

        let todo_list2: SerializableTaskList =
            autosurgeon::hydrate(&doc).expect("hydrate must succeed");
        let todo_list2: TaskList = todo_list2.into();
        assert_eq!(task_list, todo_list2);
    }
}
