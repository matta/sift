//! Persistence layer

pub(crate) use document::{load_tasks, save_tasks, Task, TaskList};

mod container;
mod crc;
mod document;
mod serialization;
