use std::fs::File;

use anyhow::Result;
use autosurgeon::{reconcile::NoKey, Reconciler};
use ratatui::widgets::ListState;
use tui_prompts::TextState;

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
    pub items: Vec<Todo>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SerializableNaiveDate(chrono::naive::NaiveDate);

impl SerializableNaiveDate {
    pub fn from_naive_date(d: chrono::naive::NaiveDate) -> Self {
        SerializableNaiveDate(d)
    }
}

impl autosurgeon::Reconcile for SerializableNaiveDate {
    type Key<'a> = NoKey;

    fn reconcile<R: Reconciler>(&self, mut reconciler: R) -> Result<(), R::Error> {
        reconciler.str(&self.0.format("%F").to_string())
    }
}

pub(crate) struct Todo {
    pub title: String,
    pub done: bool,
    pub snoozed: Option<SerializableNaiveDate>,
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
            items: (1..=10)
                .map(|i| Todo {
                    title: format!("Item {}", i),
                    done: false,
                    snoozed: None,
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
        todo!("save to file")
    }

    pub fn load(filename: &str) -> Result<App> {
        let file = File::open(filename)?;
        todo!("load from file")
        // Ok(App {
        //     state,
        //     should_quit: false,
        // })
    }
}
