use std::path::{Path, PathBuf};

use eframe::egui::{self, ScrollArea};
use sift_persist::MemoryStore;
use sift_state::State;

pub struct App {
    state: State,
    save_path: PathBuf,
}

impl App {
    pub fn load(path: &Path) -> anyhow::Result<App> {
        Ok(Self {
            state: State::new(MemoryStore::load(path)?),
            save_path: path.to_path_buf(),
        })
    }

    fn sift_save(&self) {
        // TODO: there is an eframe::App::save() method that provides a key/value
        // storage API. Consider using that here.
        self.state
            .store
            .save(&self.save_path)
            .expect("TODO: handle error");
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Todos");

            let tasks = self.state.list_tasks_for_display();
            ScrollArea::vertical().show(ui, |ui| {
                for task in tasks.iter() {
                    let checked = task.completed().is_some();
                    let mut checkbox_checked = checked;
                    ui.checkbox(&mut checkbox_checked, task.title());
                    if checkbox_checked != checked {
                        self.state.toggle_id(&task.id());
                        self.sift_save();
                    }
                }
            });
        });
    }
}
