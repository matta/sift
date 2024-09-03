use std::sync::LazyLock;

use iced::widget::{
    button, center, checkbox, column, container, keyed_column, row, scrollable, text, text_input,
};
use iced::Alignment::Center;
use iced::Element;
use iced::Length::Fill;
use sift_core::save_name;
use sift_persist::{MemoryStore, Store as _, Task, TaskId};
use sift_state::State;

pub struct App {
    loaded: Option<LoadedApp>,
}

#[derive(Debug)]
pub enum AppMessage {
    SwitchToLoaded(anyhow::Result<MemoryStore>),
    Loaded(LoadedMessage),
}

#[derive(Debug, Clone)]
pub enum LoadedMessage {
    // TODO: make this message specific to LoadedApp.
    CompleteToggled(TaskId, bool),
    CreateTaskInputChanged(String),
    CreateTask,
    Delete(TaskId),
}

impl App {
    pub fn new() -> (Self, iced::Task<AppMessage>) {
        (
            Self { loaded: None },
            iced::Task::perform(App::load(), AppMessage::SwitchToLoaded),
        )
    }

    async fn load() -> anyhow::Result<MemoryStore> {
        MemoryStore::load(&save_name())
    }

    pub fn update(&mut self, message: AppMessage) {
        match (&mut self.loaded, message) {
            (None, AppMessage::SwitchToLoaded(result)) => match result {
                Ok(store) => {
                    self.loaded = Some(LoadedApp {
                        create_task_name: String::new(),
                        state: State::new(store),
                    })
                }
                Err(_) => unimplemented!("report error and/or implement default state"),
            },
            (Some(loaded), AppMessage::Loaded(message)) => loaded.update(message),
            (None, AppMessage::Loaded(_)) => unreachable!(),
            (Some(_), AppMessage::SwitchToLoaded(_)) => unreachable!(),
        }
    }

    pub fn view(&self) -> Element<AppMessage> {
        match &self.loaded {
            None => self.view_loading(),
            Some(loaded) => Element::from(loaded.view()).map(AppMessage::Loaded),
        }
    }

    fn view_loading(&self) -> Element<AppMessage> {
        center(text("Loading...").width(Fill).align_x(Center).size(50)).into()
    }
}

pub struct LoadedApp {
    create_task_name: String,
    state: State,
}

static INPUT_ID: LazyLock<text_input::Id> = LazyLock::new(text_input::Id::unique);

impl LoadedApp {
    fn view(&self) -> Element<LoadedMessage> {
        let title = text("todos")
            .width(Fill)
            .size(100)
            .color([0.5, 0.5, 0.5])
            .align_x(Center);

        let input = text_input("What needs to be done?", &self.create_task_name)
            .id(INPUT_ID.clone())
            .on_input(LoadedMessage::CreateTaskInputChanged)
            .on_submit(LoadedMessage::CreateTask)
            .padding(15)
            .size(30)
            .align_x(Center);

        let tasks = self.state.list_tasks_for_display();
        let tasks: Element<_> = if tasks.is_empty() {
            unimplemented!("implement display of zero tasks; see https://github.com/iced-rs/iced/blob/9b99b932bced46047ec2e18c2b6ec5a6c5b3636f/examples/todos/src/main.rs#L229");
        } else {
            keyed_column(tasks.iter().map(|task| {
                let id = task.id();
                let checkbox = checkbox(task.title().to_string(), task.completed().is_some())
                    .on_toggle(move |complete| LoadedMessage::CompleteToggled(id, complete));
                let delete = button("Delete").on_press_with(move || LoadedMessage::Delete(id));
                let row = row![checkbox, delete];

                (task.id(), row.into())
            }))
            .into()
        };

        let content = column![title, input, tasks];
        scrollable(container(content).center_x(Fill).padding(20)).into()
    }

    fn update(&mut self, message: LoadedMessage) {
        match message {
            LoadedMessage::CompleteToggled(id, checked) => {
                // TODO: add a method to state that sets completion given a bool.
                if let Some(task) = self.state.get_task(&id) {
                    if task.completed().is_some() != checked {
                        self.state.toggle_id(&id);
                        self.save();
                    }
                }
            }
            LoadedMessage::CreateTaskInputChanged(s) => {
                self.create_task_name = s;
            }
            LoadedMessage::CreateTask => {
                let task = Task::new(
                    Task::new_id(),
                    std::mem::take(&mut self.create_task_name),
                    None,
                    None,
                    None,
                );
                self.state
                    .store
                    .with_transaction(|txn| {
                        let previous_task = None;
                        txn.insert_task(previous_task, &task)
                    })
                    .expect("FIXME: handle error");
            }
            LoadedMessage::Delete(id) => {
                self.state.delete_task(&id);
            }
        }
    }

    fn save(&self) {
        // TODO: don't use unwrap() below.
        self.state.store.save(&save_name()).unwrap();
    }
}
