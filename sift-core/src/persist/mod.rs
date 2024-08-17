//! Persistence layer

pub use document::{load_tasks, save_tasks, TaskList};
pub use task::{Task, TaskId};

pub use self::store::memory::MemoryStore;
pub use self::store::{Store, Transaction};

mod container;
mod crc;
mod document;
mod serialization;
mod store;
mod task;
