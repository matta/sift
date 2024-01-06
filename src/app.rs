#[derive(Default)]
pub struct App {
    pub counter: i64,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        App::default()
    }
}
