use super::{Task, TaskId};

pub trait Transaction {
    fn get_task(&self, id: &TaskId) -> anyhow::Result<Task>;

    fn put_task(&mut self, task: &Task) -> anyhow::Result<()>;

    fn delete_task(&mut self, id: &TaskId) -> anyhow::Result<()>;

    fn insert_task(&mut self, previous: Option<&TaskId>, task: &Task) -> anyhow::Result<()>;

    fn move_task(&mut self, previous: Option<&TaskId>, task: &TaskId) -> anyhow::Result<()>;

    // Commit and consume the transaction.
    //
    // See https://stackoverflow.com/q/46620790 for why this argument
    // is boxed.
    fn commit(self: Box<Self>) -> anyhow::Result<()>;
}

pub trait Store {
    fn get_task(&self, id: &TaskId) -> anyhow::Result<Task>;

    fn list_tasks(&self) -> anyhow::Result<Vec<Task>>;

    fn undo(&mut self) -> anyhow::Result<()>;

    fn redo(&mut self) -> anyhow::Result<()>;

    fn transaction<'a>(&'a mut self) -> Box<dyn Transaction + 'a>;

    fn with_transaction<F>(&mut self, callback: F) -> anyhow::Result<()>
    where
        F: FnOnce(&mut dyn Transaction) -> anyhow::Result<()>,
    {
        let mut txn = self.transaction();
        callback(txn.as_mut())?;
        txn.commit()?;
        Ok(())
    }
}

pub(crate) mod memory {
    use std::fmt;
    use std::path::Path;

    use anyhow::bail;

    use super::{Store, Transaction};
    use crate::{load_tasks, save_tasks, Task, TaskId, TaskList};

    #[derive(Default, Clone)]
    struct Record {
        tasks: im::HashMap<TaskId, Task>,
        order: im::Vector<TaskId>,
    }

    impl Record {
        fn get_task(&self, id: &TaskId) -> Result<Task, anyhow::Error> {
            self.tasks
                .get(id)
                .map_or_else(|| bail!("task not found"), |task| Ok(task.clone()))
        }

        fn put_task(&mut self, task: &Task) {
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
        }

        fn insert_task(&mut self, previous: Option<&TaskId>, task: &Task) {
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
        }

        fn delete_task(&mut self, id: &TaskId) {
            self.order.retain(|entry| entry != id);
            self.tasks.retain(|key, _| key != id);
        }

        fn move_task(&mut self, previous: Option<&TaskId>, id: &TaskId) {
            self.order.retain(|other| other != id);

            let index = if let Some(previous_id) = previous {
                self.order.index_of(previous_id).map(|index| index + 1)
            } else {
                None
            }
            .unwrap_or(0);

            self.order.insert(index, *id);
        }

        fn list_tasks(&self) -> Vec<Task> {
            self.order
                .iter()
                .map(|id| {
                    self.tasks
                        .get(id)
                        .expect("all items in MemoryStore::order must be in MemoryStore::tasks")
                        .clone()
                })
                .collect()
        }
    }

    #[derive(Default)]
    pub struct MemoryStore {
        current: Record,
        undo_stack: Vec<Record>,
        redo_stack: Vec<Record>,
    }

    impl fmt::Debug for MemoryStore {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("MemoryStore")
                .field("current", &"Record")
                .field("undo_stack", &format_args!("Vec<Record>"))
                .field("redo_stack", &format_args!("Vec<Record>"))
                .finish()
        }
    }

    struct MemoryTransaction<'a> {
        store: &'a mut MemoryStore,
        start: Record,
    }

    impl<'a> MemoryTransaction<'a> {
        fn new(store: &'a mut MemoryStore) -> Self {
            let start = store.current.clone();
            Self { store, start }
        }
    }

    impl Transaction for MemoryTransaction<'_> {
        fn get_task(&self, id: &TaskId) -> anyhow::Result<Task> {
            self.store.get_task_impl(id)
        }

        fn put_task(&mut self, task: &Task) -> anyhow::Result<()> {
            self.store.put_task_impl(task);
            Ok(())
        }

        fn delete_task(&mut self, id: &TaskId) -> anyhow::Result<()> {
            self.store.delete_task(id);
            Ok(())
        }

        fn insert_task(&mut self, previous: Option<&TaskId>, task: &Task) -> anyhow::Result<()> {
            self.store.insert_task(previous, task);
            Ok(())
        }

        fn move_task(&mut self, previous: Option<&TaskId>, task: &TaskId) -> anyhow::Result<()> {
            self.store.move_task(previous, task);
            Ok(())
        }

        fn commit(self: Box<Self>) -> anyhow::Result<()> {
            self.store.undo_stack.push(self.start);
            Ok(())
        }
    }

    impl MemoryStore {
        #[allow(dead_code)]
        pub(crate) fn new() -> Self {
            Self::default()
        }

        pub fn load(path: &Path) -> Result<MemoryStore, anyhow::Error> {
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

        pub fn save(&self, path: &Path) -> Result<(), anyhow::Error> {
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

        fn delete_task(&mut self, id: &TaskId) {
            self.current.delete_task(id);
        }

        fn insert_task(&mut self, previous: Option<&TaskId>, task: &Task) {
            let saved = self.current.clone();
            self.current.insert_task(previous, task);
            self.undo_stack.push(saved);
        }

        fn move_task(&mut self, previous: Option<&TaskId>, task: &TaskId) {
            let saved = self.current.clone();
            self.current.move_task(previous, task);
            self.undo_stack.push(saved);
        }

        fn get_task_impl(&self, id: &TaskId) -> anyhow::Result<Task> {
            self.current.get_task(id)
        }

        fn put_task_impl(&mut self, task: &Task) {
            self.current.put_task(task);
        }
    }

    impl Store for MemoryStore {
        fn get_task(&self, id: &TaskId) -> anyhow::Result<Task> {
            self.get_task_impl(id)
        }

        fn list_tasks(&self) -> anyhow::Result<Vec<Task>> {
            let tasks = self.current.list_tasks();
            Ok(tasks)
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

        fn transaction<'a>(&'a mut self) -> Box<dyn Transaction + 'a> {
            let transaction = MemoryTransaction::new(self);
            Box::new(transaction)
        }
    }
}
