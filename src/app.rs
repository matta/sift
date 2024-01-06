use std::fs::File;

use anyhow::Result;
use ratatui::widgets::ListState;
use serde::{Deserialize, Serialize};

#[derive(Default)]
pub(crate) struct App {
    pub should_quit: bool,
    pub list: TodoList,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct TodoList {
    #[serde(with = "SerializableListState")]
    pub state: ListState,
    pub items: Vec<Todo>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Todo {
    pub title: String,
    pub done: bool,
}

/// Bridges [ListState] to a serializable struct.
///
/// The `ratatui` `ListState` struct is not serializable.
/// This struct is structually identialto `ListState` and is serializable.
///
/// # Example
///
/// To make a `ListState` field serializable, declare it like this:
///
/// ```
/// struct MyStruct {
///     #[serde(with "SerializableListState")]    
///     state: ListState
/// }
#[derive(Serialize, Deserialize)]
#[serde(remote = "ListState")]
struct SerializableListState {
    #[serde(getter = "ListState::offset")]
    offset: usize,
    #[serde(getter = "ListState::selected")]
    selected: Option<usize>,
}

/// Serde deserialization uses this to convert a `SerializableListState` into
/// a `ListState`.
impl From<SerializableListState> for ListState {
    fn from(from: SerializableListState) -> ListState {
        ListState::default()
            .with_offset(from.offset)
            .with_selected(from.selected)
    }
}

impl Default for TodoList {
    fn default() -> Self {
        TodoList {
            state: ListState::default(),
            items: (1..=100)
                .map(|i| Todo {
                    title: format!("Item {}", i),
                    done: false,
                })
                .collect(),
        }
    }
}

impl TodoList {
    pub(crate) fn next(&mut self) {
        let i = if let Some(i) = self.state.selected() {
            (i + 1) % self.items.len()
        } else {
            0
        };
        self.state.select(Some(i));
    }

    pub(crate) fn previous(&mut self) {
        let i = if let Some(i) = self.state.selected() {
            if i == 0 {
                self.items.len() - 1
            } else {
                i - 1
            }
        } else {
            0
        };
        self.state.select(Some(i));
    }

    pub(crate) fn unselect(&mut self) {
        self.state.select(None);
    }

    pub(crate) fn toggle(&mut self) {
        if let Some(i) = self.state.selected() {
            self.items[i].done = !self.items[i].done;
        }
    }
}

impl App {
    pub fn new() -> Self {
        App::default()
    }

    pub fn save(self: &App, filename: &str) -> Result<()> {
        let file = File::create(filename)?;
        serde_json::to_writer_pretty(file, &self.list)?;
        Ok(())
    }

    pub fn load(filename: &str) -> Result<App> {
        let file = File::open(filename)?;
        let list = serde_json::from_reader(file)?;
        Ok(App {
            list,
            should_quit: false,
        })
    }
}
