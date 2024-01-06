use ratatui::{
    prelude::Frame,
    widgets::{Block, Borders, List, ListItem},
};

use crate::app::{App, Screen};

pub(crate) fn render(app: &mut App, f: &mut Frame) {
    match &app.state.screen {
        Screen::Main => {
            let items: Vec<_> = app.state.list.items.iter().map(render_todo).collect();
            let items = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("List"))
                .highlight_symbol("> ");

            f.render_stateful_widget(items, f.size(), &mut app.state.list.state);
        }
        Screen::Edit(edit_state) => f.render_widget(edit_state.textarea.widget(), f.size()),
    }
}

fn render_todo(s: &crate::app::Todo) -> ListItem<'_> {
    let check = if s.done { 'x' } else { ' ' };
    ListItem::new(format!("[{}] {}", check, s.title.as_str()))
}
