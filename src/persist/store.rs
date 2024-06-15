use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::path::Path;

use im::Vector;

use super::{load_tasks, save_tasks, Task, TaskId, TaskList};

pub(crate) trait Store {
    // FIXME: return Result
    #[must_use]
    fn get_task(&mut self, id: &TaskId) -> Option<Task>;

    // TODO: rename to put_task
    fn put_task(&mut self, task: &Task) -> anyhow::Result<()>;

    // TODO: rename to insert_task
    fn insert_task(&mut self, previous: Option<&TaskId>, task: &Task) -> anyhow::Result<()>;

    // TODO: rename to delete_task
    fn delete_task(&mut self, id: &TaskId) -> anyhow::Result<()>;

    // If prevous is None moves to the front, otherwise moves after previous.
    fn move_task(&mut self, previous: Option<&TaskId>, task: &TaskId) -> anyhow::Result<()>;

    // TODO: rename to set_task_title
    fn set_title(&mut self, id: &TaskId, title: &str);
}

#[derive(Default)]
pub(crate) struct MemoryStore {
    tasks: HashMap<TaskId, Task>,
    order: Vector<TaskId>,
}

impl MemoryStore {
    #[allow(dead_code)]
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn load(path: &Path) -> Result<MemoryStore, anyhow::Error> {
        let tasks = load_tasks(path)?;

        let order: Vector<TaskId> = tasks.tasks.iter().map(Task::id).collect();
        let tasks: HashMap<TaskId, Task> = tasks
            .tasks
            .into_iter()
            .map(|task| (task.id(), task))
            .collect();

        Ok(MemoryStore { tasks, order })
    }

    pub(crate) fn save(&self, path: &Path) -> Result<(), anyhow::Error> {
        let tasks = TaskList {
            tasks: self
                .order
                .iter()
                .map(|id| self.tasks.get(id).unwrap())
                .cloned()
                .collect(),
        };
        save_tasks(path, &tasks)?;
        Ok(())
    }

    pub(crate) fn len(&self) -> usize {
        self.order.len()
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &Task> {
        self.order.iter().map(|id| {
            self.tasks
                .get(id)
                .expect("all items in MemoryStore::order must be in MemoryStore::tasks")
        })
    }
}

impl Store for MemoryStore {
    fn get_task(&mut self, id: &TaskId) -> Option<Task> {
        self.tasks.get(id).cloned()
    }

    fn put_task(&mut self, task: &Task) -> anyhow::Result<()> {
        debug_assert!(
            self.order.contains(&task.id()),
            "MemoryStore::put called with task not in the order list"
        );
        match self.tasks.entry(task.id()) {
            Entry::Occupied(mut entry) => *entry.get_mut() = task.clone(),
            Entry::Vacant(_) => {
                panic!("MemoryStore::put called with task not in the tasks map")
            }
        }

        Ok(())
    }

    fn insert_task(&mut self, previous: Option<&TaskId>, task: &Task) -> anyhow::Result<()> {
        let index = if let Some(previous) = previous {
            self.order
                .index_of(previous)
                .map_or(0, |index| index.saturating_add(1))
        } else {
            0
        };

        debug_assert!(
            !self.order.contains(&task.id()),
            "MemoryStore::insert called with task.id already in the order list"
        );
        self.order.insert(index, task.id());
        self.tasks.entry(task.id()).or_insert(task.clone());

        Ok(())
    }

    fn delete_task(&mut self, id: &TaskId) -> anyhow::Result<()> {
        self.order.retain(|entry| entry != id);
        self.tasks.retain(|key, _| key != id);
        Ok(())
    }

    fn set_title(&mut self, id: &TaskId, title: &str) {
        self.tasks.get_mut(id).unwrap().set_title(title.to_string());
    }

    fn move_task(&mut self, previous: Option<&TaskId>, id: &TaskId) -> anyhow::Result<()> {
        self.order.retain(|other| other != id);

        let index = if let Some(previous_id) = previous {
            self.order.index_of(previous_id).map(|index| index + 1)
        } else {
            None
        }
        .unwrap_or(0);

        self.order.insert(index, *id);

        Ok(())
    }
}
