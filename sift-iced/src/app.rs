use std::sync::LazyLock;

use iced::widget::{
    center, checkbox, column, container, keyed_column, row, scrollable, text, text_input,
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

// Note: this message is Clone because text_input requires it. The top level
// Message enum is not clone because it contains a MemoryStore, which is
// not clone. Another way to go would be to use Arc, somehow.
#[derive(Debug, Clone)]
pub enum CreateMessage {
    InputChanged(String),
    CreateTask,
}

#[derive(Debug)]
pub enum Message {
    Loaded(anyhow::Result<MemoryStore>),
    // TODO: make this message specific to LoadedApp.
    CompleteToggled(TaskId, bool),
    CreateTask(CreateMessage),
}

impl App {
    pub fn new() -> (Self, iced::Task<Message>) {
        (
            Self { loaded: None },
            iced::Task::perform(App::load(), Message::Loaded),
        )
    }

    async fn load() -> anyhow::Result<MemoryStore> {
        MemoryStore::load(&save_name())
    }

    pub fn update(&mut self, message: Message) {
        match &mut self.loaded {
            None => self.update_loading(message),
            Some(loaded) => loaded.update(message),
        }
    }

    fn update_loading(&mut self, message: Message) {
        match message {
            Message::Loaded(result) => match result {
                Ok(store) => {
                    self.loaded = Some(LoadedApp {
                        create_name: String::new(),
                        state: State::new(store),
                    })
                }
                Err(_) => unimplemented!("report error and/or implement default state"),
            },
            Message::CompleteToggled(_, _) => {
                unreachable!()
            }
            Message::CreateTask(_) => todo!(),
        }
    }

    pub fn view(&self) -> Element<Message> {
        match &self.loaded {
            None => self.view_loading(),
            Some(loaded) => loaded.view(),
        }
    }

    fn view_loading(&self) -> Element<Message> {
        center(text("Loading...").width(Fill).align_x(Center).size(50)).into()
    }
}

pub struct LoadedApp {
    create_name: String,
    state: State,
}

static INPUT_ID: LazyLock<text_input::Id> = LazyLock::new(text_input::Id::unique);

impl LoadedApp {
    fn view(&self) -> Element<Message> {
        let title = text("todos")
            .width(Fill)
            .size(100)
            .color([0.5, 0.5, 0.5])
            .align_x(Center);

        let input = Element::from(
            text_input("What needs to be done?", &self.create_name)
                .id(INPUT_ID.clone())
                .on_input(CreateMessage::InputChanged)
                .on_submit(CreateMessage::CreateTask)
                .padding(15)
                .size(30),
            //.align_x(Center);
        )
        .map(Message::CreateTask);

        let tasks = self.state.list_tasks_for_display();
        let tasks: Element<_> = if tasks.is_empty() {
            unimplemented!("implement display of zero tasks; see https://github.com/iced-rs/iced/blob/9b99b932bced46047ec2e18c2b6ec5a6c5b3636f/examples/todos/src/main.rs#L229");
        } else {
            keyed_column(tasks.iter().map(|task| {
                let id = task.id();
                let checkbox = checkbox(task.title().to_string(), task.completed().is_some())
                    .on_toggle(move |complete| Message::CompleteToggled(id, complete));
                let row = row![checkbox];

                (task.id(), row.into())
            }))
            .into()
        };

        let content = column![title, input, tasks];
        scrollable(container(content).center_x(Fill).padding(20)).into()
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Loaded(_) => unreachable!(),
            Message::CompleteToggled(id, checked) => {
                // TODO: add a method to state that sets completion given a bool.
                if let Some(task) = self.state.get_task(&id) {
                    if task.completed().is_some() != checked {
                        self.state.toggle_id(&id);
                        self.save();
                    }
                }
            }
            Message::CreateTask(msg) => match msg {
                CreateMessage::InputChanged(s) => {
                    self.create_name = s;
                }
                CreateMessage::CreateTask => {
                    let task = Task::new(
                        Task::new_id(),
                        std::mem::take(&mut self.create_name),
                        None,
                        None,
                        None,
                    );
                    self.state
                        .store
                        .with_transaction(|txn| {
                            let previous = None;
                            txn.insert_task(previous, &task)
                        })
                        .expect("FIXME: handle error");
                }
            },
        }
    }

    fn save(&self) {
        // TODO: don't use unwrap() below.
        self.state.store.save(&save_name()).unwrap();
    }
}
