use std::path::Path;

use eframe::egui::{self, vec2, ScrollArea, TextStyle};

use sift_persist::MemoryStore;
use sift_state::State;

pub struct App {
    state: State,
}

impl App {
    pub fn load(path: &Path) -> anyhow::Result<App> {
        Ok(Self {
            state: State::new(MemoryStore::load(path)?),
        })
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Todos");

            let tasks = self.state.list_tasks_for_display();
            let height = TextStyle::Body.resolve(ui.style()).size;
            ScrollArea::vertical().show_rows(ui, height, tasks.len(), |ui, row_range| {
                ui.allocate_space(vec2(ui.available_width(), 0.0));
                for (index, task) in tasks.iter().enumerate() {
                    if !row_range.contains(&index) {
                        continue;
                    }
                    let checked = task.completed().is_some();
                    let mut checkbox_checked = checked;
                    ui.checkbox(&mut checkbox_checked, task.title());
                    if checkbox_checked != checked {
                        self.state.toggle_id(task.id());
                    }
                }
            });
        });
    }
}
