use std::fmt::Display;

use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct TaskId(Uuid);

impl Display for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<Uuid> for TaskId {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Task {
    /// Task identifier.
    id: TaskId,

    /// Title of the task.
    title: String,

    /// Snooze date of the task.  Tasks with a snoozed date do not appear
    // by default if the date is before the current date.
    snoozed: Option<NaiveDate>,

    /// Due date of the task.
    due: Option<NaiveDate>,

    /// Completion date and time of the task.  If `None`, the task is
    /// incomplete.
    completed: Option<DateTime<Utc>>,
}

impl Task {
    pub(crate) fn new(
        id: TaskId,
        title: String,
        snoozed: Option<NaiveDate>,
        due: Option<NaiveDate>,
        completed: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id,
            title,
            snoozed,
            due,
            completed,
        }
    }

    pub(crate) fn new_id() -> TaskId {
        let context = uuid::NoContext;
        let ts = uuid::Timestamp::now(context);
        TaskId(uuid::Uuid::new_v7(ts))
    }

    pub(crate) fn id(&self) -> TaskId {
        self.id
    }

    pub(crate) fn title(&self) -> &str {
        &self.title
    }

    pub(crate) fn snoozed(&self) -> Option<NaiveDate> {
        self.snoozed
    }

    pub(crate) fn due(&self) -> Option<NaiveDate> {
        self.due
    }

    pub(crate) fn completed(&self) -> Option<DateTime<Utc>> {
        self.completed
    }

    pub(crate) fn set_title(&mut self, title: String) {
        self.title = title;
    }

    pub(crate) fn set_snoozed(&mut self, snoozed: Option<NaiveDate>) {
        self.snoozed = snoozed;
    }

    pub(crate) fn set_completed(&mut self, completed: Option<DateTime<Utc>>) {
        self.completed = completed;
    }
}
