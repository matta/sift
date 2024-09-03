use sift_iced::App;

pub fn main() -> iced::Result {
    iced::application("Sift", App::update, App::view)
        .font(iced_aw::BOOTSTRAP_FONT_BYTES)
        .run_with(App::new)
}
