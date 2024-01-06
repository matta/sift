use ratatui::{
    prelude::Frame,
    widgets::{Block, Borders, List, ListItem},
};

use crate::app::App;

pub(crate) fn render(app: &mut App, f: &mut Frame) {
    let items: Vec<_> = app.list.items.iter().map(render_todo).collect();
    let items = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("List"))
        .highlight_symbol("> ");

    f.render_stateful_widget(items, f.size(), &mut app.list.state);
}

fn render_todo(s: &crate::app::Todo) -> ListItem<'_> {
    let check = if s.done { 'x' } else { ' ' };
    ListItem::new(format!("[{}] {}", check, s.title.as_str()))
}
