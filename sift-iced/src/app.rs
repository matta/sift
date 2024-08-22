use iced::widget::{center, checkbox, column, container, keyed_column, row, scrollable, text};
use iced::Alignment::Center;
use iced::Element;
use iced::Length::Fill;
use sift_core::save_name;
use sift_persist::{MemoryStore, TaskId};
use sift_state::State;

pub struct App {
    loaded: Option<LoadedApp>,
}

#[derive(Debug)]
pub enum Message {
    Loaded(anyhow::Result<MemoryStore>),
    // TODO: make this message specific to LoadedApp.
    CompleteToggled(TaskId, bool),
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
                        state: State::new(store),
                    })
                }
                Err(_) => unimplemented!("report error and/or implement default state"),
            },
            Message::CompleteToggled(_, _) => {
                unreachable!()
            }
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
    state: State,
}

impl LoadedApp {
    fn view(&self) -> Element<Message> {
        let title = text("todos")
            .width(Fill)
            .size(100)
            .color([0.5, 0.5, 0.5])
            .align_x(Center);
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

        let content = column![title, tasks];
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
        }
    }

    fn save(&self) {
        // TODO: don't use unwrap() below.
        self.state.store.save(&save_name()).unwrap();
    }
}
