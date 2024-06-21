//! Persistence layer

pub(crate) use document::{load_tasks, save_tasks, TaskList};
pub(crate) use task::{Task, TaskId};

pub(crate) use self::store::memory::MemoryStore;
pub(crate) use self::store::Store;

mod container;
mod crc;
mod document;
mod serialization;
mod store;
mod task;
