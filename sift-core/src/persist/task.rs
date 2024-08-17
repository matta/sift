use std::fmt::Display;

use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TaskId(Uuid);

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
pub struct Task {
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
    #[must_use]
    pub fn new(
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

    #[must_use]
    pub fn new_id() -> TaskId {
        let context = uuid::NoContext;
        let ts = uuid::Timestamp::now(context);
        TaskId(uuid::Uuid::new_v7(ts))
    }

    #[must_use]
    pub fn id(&self) -> TaskId {
        self.id
    }

    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    #[must_use]
    pub fn snoozed(&self) -> Option<NaiveDate> {
        self.snoozed
    }

    #[must_use]
    pub fn due(&self) -> Option<NaiveDate> {
        self.due
    }

    #[must_use]
    pub fn completed(&self) -> Option<DateTime<Utc>> {
        self.completed
    }

    pub fn set_title(&mut self, title: String) {
        self.title = title;
    }

    pub fn set_snoozed(&mut self, snoozed: Option<NaiveDate>) {
        self.snoozed = snoozed;
    }

    pub fn set_completed(&mut self, completed: Option<DateTime<Utc>>) {
        self.completed = completed;
    }
}
