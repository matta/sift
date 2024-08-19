use std::{iter, path::Path};

use eframe::egui::{self, vec2, ScrollArea, TextStyle};

use itertools::Itertools as _;
use sift_persist::MemoryStore;

use crate::state::State;

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
            let mut checked = tasks
                .iter()
                .map(|task| task.completed().is_some())
                .collect_vec();
            let height = TextStyle::Body.resolve(ui.style()).size;
            ScrollArea::vertical().show_rows(ui, height, tasks.len(), |ui, row_range| {
                ui.allocate_space(vec2(ui.available_width(), 0.0));
                for (index, (task, checked)) in
                    iter::zip(tasks.iter(), checked.iter_mut()).enumerate()
                {
                    if !row_range.contains(&index) {
                        continue;
                    }
                    if ui.checkbox(checked, task.title()).changed()
                        && *checked != task.completed().is_some()
                    {
                        self.state.toggle_id(task.id());
                    }
                }
            });
        });
    }
}
