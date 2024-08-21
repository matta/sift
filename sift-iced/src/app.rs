use iced::widget::{center, column, container, keyed_column, row, scrollable, text};
use iced::Alignment::Center;
use iced::Element;
use iced::Length::Fill;
use sift_persist::MemoryStore;
use sift_state::State;

pub enum App {
    Loading,
    Loaded(State),
}

#[derive(Debug)]
pub enum Message {
    Loaded(anyhow::Result<MemoryStore>),
}

impl App {
    pub fn new() -> (Self, iced::Task<Message>) {
        (
            App::Loading,
            iced::Task::perform(App::load(), Message::Loaded),
        )
    }

    async fn load() -> anyhow::Result<MemoryStore> {
        let path = sift_core::save_name();
        MemoryStore::load(&path)
        //     unwrap();
        // let state = State::new(store);

        // Ok(Self {
        //     state: State::new(MemoryStore::load(&path)?),
        // })
    }

    pub fn update(&mut self, message: Message) {
        match self {
            App::Loading => {
                self.update_loading(message);
            }
            App::Loaded(_) => self.update_loaded(),
        }
    }

    fn update_loading(&mut self, message: Message) {
        match message {
            Message::Loaded(result) => match result {
                Ok(store) => *self = App::Loaded(State::new(store)),
                Err(_) => unimplemented!("report error and/or implement default state"),
            },
        }
    }

    fn update_loaded(&mut self) {
        todo!()
    }

    pub fn view(&self) -> Element<Message> {
        match self {
            App::Loading => self.view_loading(),
            App::Loaded(state) => self.view_loaded(state),
        }
    }

    fn view_loading(&self) -> Element<Message> {
        center(text("Loading...").width(Fill).align_x(Center).size(50)).into()
    }

    fn view_loaded(&self, state: &State) -> Element<Message> {
        let title = text("todos")
            .width(Fill)
            .size(100)
            .color([0.5, 0.5, 0.5])
            .align_x(Center);
        let tasks = state.list_tasks_for_display();
        let tasks: Element<_> = if tasks.is_empty() {
            unimplemented!("implement display of zero tasks; see https://github.com/iced-rs/iced/blob/9b99b932bced46047ec2e18c2b6ec5a6c5b3636f/examples/todos/src/main.rs#L229");
        } else {
            keyed_column(tasks.iter().map(|task| {
                let text = text(task.title().to_string());
                let row = row![text];

                (task.id(), row.into())
            }))
            .into()
        };

        let content = column![title, tasks];
        scrollable(container(content).center_x(Fill).padding(20)).into()
    }
}

// impl eframe::App for App {
//     fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
//         egui::CentralPanel::default().show(ctx, |ui| {
//             ui.heading("Todos");

//             let tasks = self.state.list_tasks_for_display();
//             let height = TextStyle::Body.resolve(ui.style()).size;
//             ScrollArea::vertical().show_rows(ui, height, tasks.len(), |ui,
// row_range| {                 ui.allocate_space(vec2(ui.available_width(),
// 0.0));                 for (index, task) in tasks.iter().enumerate() {
//                     if !row_range.contains(&index) {
//                         continue;
//                     }
//                     let checked = task.completed().is_some();
//                     let mut checkbox_checked = checked;
//                     ui.checkbox(&mut checkbox_checked, task.title());
//                     if checkbox_checked != checked {
//                         self.state.toggle_id(task.id());
//                     }
//                 }
//             });
//         });
//     }
// }
