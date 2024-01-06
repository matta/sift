use ratatui::widgets::ListState;

#[derive(Default)]
pub struct App {
    pub should_quit: bool,
    pub list: StatefulList,
}

pub struct StatefulList {
    pub state: ListState,
    pub items: Vec<String>,
}

impl Default for StatefulList {
    fn default() -> Self {
        StatefulList {
            state: ListState::default(),
            items: (1..=100).map(|i| format!("Item {}", i)).collect(),
        }
    }
}

impl StatefulList {
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn unselect(&mut self) {
        self.state.select(None);
    }
}

impl App {
    pub fn new() -> Self {
        App::default()
    }
}
