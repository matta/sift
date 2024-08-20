use iced::widget::{button, column, text, Column};

#[derive(Default)]
pub struct App {
    value: i64,
}

#[derive(Debug, Clone)]
pub enum Message {
    Increment,
    Decrement,
}

impl App {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::Increment => {
                self.value += 1;
            }
            Message::Decrement => {
                self.value -= 1;
            }
        }
    }

    pub fn view(&self) -> Column<Message> {
        column![
            button("+").on_press(Message::Increment),
            text(self.value),
            button("-").on_press(Message::Decrement),
        ]
    }
}
