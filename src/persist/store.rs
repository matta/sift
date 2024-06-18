use std::path::Path;

use anyhow::bail;

use super::{load_tasks, save_tasks, Task, TaskId, TaskList};

pub(crate) trait Store {
    fn get_task(&mut self, id: &TaskId) -> anyhow::Result<Task>;

    fn put_task(&mut self, task: &Task) -> anyhow::Result<()>;

    fn insert_task(&mut self, previous: Option<&TaskId>, task: &Task) -> anyhow::Result<()>;

    fn delete_task(&mut self, id: &TaskId) -> anyhow::Result<()>;

    // If prevous is None moves to the front, otherwise moves after previous.
    fn move_task(&mut self, previous: Option<&TaskId>, task: &TaskId) -> anyhow::Result<()>;

    fn list_tasks(&mut self) -> anyhow::Result<Vec<Task>>;

    fn undo(&mut self) -> anyhow::Result<()>;

    fn redo(&mut self) -> anyhow::Result<()>;
}

#[derive(Default, Clone)]
struct Record {
    tasks: im::HashMap<TaskId, Task>,
    order: im::Vector<TaskId>,
}

impl Record {
    fn get_task(&mut self, id: &TaskId) -> Result<Task, anyhow::Error> {
        self.tasks
            .get(id)
            .map_or_else(|| bail!("task not found"), |task| Ok(task.clone()))
    }

    fn put_task(&mut self, task: &Task) -> anyhow::Result<()> {
        use im::hashmap::Entry::{Occupied, Vacant};
        debug_assert!(
            self.order.contains(&task.id()),
            "MemoryStore::put called with task not in the order list"
        );
        match self.tasks.entry(task.id()) {
            Occupied(mut entry) => *entry.get_mut() = task.clone(),
            Vacant(_) => {
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

    fn list_tasks(&mut self) -> anyhow::Result<Vec<Task>> {
        Ok(self
            .order
            .iter()
            .map(|id| {
                self.tasks
                    .get(id)
                    .expect("all items in MemoryStore::order must be in MemoryStore::tasks")
                    .clone()
            })
            .collect())
    }
}

#[derive(Default)]
pub(crate) struct MemoryStore {
    current: Record,
    undo_stack: Vec<Record>,
    redo_stack: Vec<Record>,
}

impl MemoryStore {
    #[allow(dead_code)]
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn load(path: &Path) -> Result<MemoryStore, anyhow::Error> {
        let tasks = load_tasks(path)?;

        let order: im::Vector<TaskId> = tasks.tasks.iter().map(Task::id).collect();
        let tasks: im::HashMap<TaskId, Task> = tasks
            .tasks
            .into_iter()
            .map(|task| (task.id(), task))
            .collect();

        Ok(MemoryStore {
            current: Record { tasks, order },
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        })
    }

    pub(crate) fn save(&self, path: &Path) -> Result<(), anyhow::Error> {
        let tasks = TaskList {
            tasks: self
                .current
                .order
                .iter()
                .map(|id| self.current.tasks.get(id).unwrap())
                .cloned()
                .collect(),
        };
        save_tasks(path, &tasks)?;
        Ok(())
    }
}

impl Store for MemoryStore {
    fn get_task(&mut self, id: &TaskId) -> anyhow::Result<Task> {
        self.current.get_task(id)
    }

    fn put_task(&mut self, task: &Task) -> anyhow::Result<()> {
        let saved = self.current.clone();
        self.current.put_task(task)?;
        self.undo_stack.push(saved);
        Ok(())
    }

    fn insert_task(&mut self, previous: Option<&TaskId>, task: &Task) -> anyhow::Result<()> {
        let saved = self.current.clone();
        self.current.insert_task(previous, task)?;
        self.undo_stack.push(saved);
        Ok(())
    }

    fn delete_task(&mut self, id: &TaskId) -> anyhow::Result<()> {
        let saved = self.current.clone();
        self.current.delete_task(id)?;
        self.undo_stack.push(saved);
        Ok(())
    }

    fn move_task(&mut self, previous: Option<&TaskId>, task: &TaskId) -> anyhow::Result<()> {
        let saved = self.current.clone();
        self.current.move_task(previous, task)?;
        self.undo_stack.push(saved);
        Ok(())
    }

    fn list_tasks(&mut self) -> anyhow::Result<Vec<Task>> {
        self.current.list_tasks()
    }

    fn undo(&mut self) -> anyhow::Result<()> {
        if let Some(record) = self.undo_stack.pop() {
            self.redo_stack.push(record.clone());
            self.current = record;
            Ok(())
        } else {
            bail!("undo is not available")
        }
    }

    fn redo(&mut self) -> anyhow::Result<()> {
        if let Some(record) = self.redo_stack.pop() {
            self.current = record;
            Ok(())
        } else {
            bail!("redo is not available")
        }
    }
}
