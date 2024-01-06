use ratatui::{
    prelude::Frame,
    widgets::{Block, Borders, List, ListItem},
};

use crate::app::App;

pub fn render(app: &mut App, f: &mut Frame) {
    let items: Vec<_> = app
        .list
        .items
        .iter()
        .map(|s| ListItem::new(s.as_str()))
        .collect();
    let items = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("List"))
        .highlight_symbol(">> ");

    f.render_stateful_widget(items, f.size(), &mut app.list.state);
}
