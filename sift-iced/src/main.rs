use sift_iced::App;

pub fn main() -> iced::Result {
    iced::application("Sift", App::update, App::view).run_with(App::new)
}
