use std::{fs::File, io::{Read, Write}};

use anyhow::Result;
use ratatui::widgets::ListState;
use tui_prompts::TextState;

use crate::persist::{Task, TaskList, encode_document, decode_document};

#[derive(Default)]
pub(crate) struct App {
    pub should_quit: bool,
    pub state: State,
}

#[derive(Default)]
pub(crate) struct State {
    pub list: TodoList,
    pub screen: Screen,
}

#[derive(Default)]
pub(crate) enum Screen {
    #[default]
    Main,
    Edit(EditState),
}

pub(crate) struct EditState {
    pub index: usize,
    // TODO: in upstream make the 'static workaround used here more
    // discoverable.  See
    // https://github.com/rhysd/tui-textarea/issues/46
    pub text_state: TextState<'static>,
}

pub(crate) struct TodoList {
    pub state: ListState,
    pub tasks: TaskList,
}

/// Bridges [ListState] to a serializable struct.
///
/// The `ratatui` `ListState` struct is not serializable.  This struct is
/// structurally identical to `ListState` and is serializable.
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
struct SerializableListState {
    offset: usize,
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
            tasks: TaskList {
                tasks: (1..=10)
                .map(|i| Task {
                    title: format!("Item {}", i),
                    snoozed: None,
                    due_date: None,
                    completed: false,
                })
                .collect()
            }
        }
    }
}

impl TodoList {
    pub(crate) fn next(&mut self) {
        let i = if let Some(i) = self.state.selected() {
            (i + 1) % self.tasks.tasks.len()
        } else {
            0
        };
        self.state.select(Some(i));
    }

    pub(crate) fn previous(&mut self) {
        let i = if let Some(i) = self.state.selected() {
            if i == 0 {
                self.tasks.tasks.len() - 1
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
            self.tasks.tasks[i].completed = !self.tasks.tasks[i].completed;
        }
    }
}

impl App {
    pub fn new() -> Self {
        App::default()
    }

    pub fn save(self: &App, filename: &str) -> Result<()> {
        let binary = encode_document(&self.state.list.tasks)?;
        let mut file = File::create(filename)?;
        file.write_all(&binary)?;
        file.sync_all()?;
        Ok(())
    }

    pub fn load(filename: &str) -> Result<App> {
        let mut file = File::open(filename)?;

        let mut binary = Vec::new();
        file.read_to_end(&mut binary)?;

        let tasks = decode_document(binary)?;

        Ok(App {
            state: State {
                list: TodoList {
                    tasks,
                    state: ListState::default(),
                },
                screen: Screen::Main,
            },
            should_quit: false,
        })
    }
}
