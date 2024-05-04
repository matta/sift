use std::borrow::Cow;

use ratatui::{
    prelude::Frame,
    widgets::{Block, Borders, List, ListItem},
};
use tui_prompts::{State, TextPrompt};

use crate::app::{App, Screen};

pub(crate) fn render(app: &mut App, f: &mut Frame) {
    match &mut app.state.screen {
        Screen::Main => {
            let items: Vec<_> = app.state.list.tasks.tasks.iter().map(render_task).collect();
            let items = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("List"))
                .highlight_symbol("> ");

            f.render_stateful_widget(items, f.size(), &mut app.state.list.state);
        }
        Screen::Edit(edit_state) => {
            let prompt = TextPrompt::new(Cow::Borrowed("edit"));
            let text_state = &mut edit_state.text_state;
            f.render_stateful_widget(prompt, f.size(), text_state);
            let (x, y) = text_state.cursor();
            f.set_cursor(x, y);
        }
    }
}

fn render_task(s: &crate::persist::Task) -> ListItem<'_> {
    let check = if s.completed {
        'x'
    } else {
        ' '
    };
    ListItem::new(format!("[{}] {}", check, s.title.as_str()))
}
