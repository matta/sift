use sift_iced::App;

pub fn main() -> iced::Result {
    iced::run("A cool counter", App::update, App::view)
}
