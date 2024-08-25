use std::path::{Path, PathBuf};

use eframe::egui::{self, Button, ScrollArea};
use sift_persist::{MemoryStore, Store as _, Task, TaskId};
use sift_state::State;

pub struct App {
    state: State,
    save_path: PathBuf,
    editing_task: Option<TaskId>,
}

impl App {
    pub fn load(path: &Path) -> anyhow::Result<App> {
        Ok(Self {
            state: State::new(MemoryStore::load(path)?),
            save_path: path.to_path_buf(),
            editing_task: None,
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

            ScrollArea::vertical().show(ui, |ui| {
                let add_task_clicked = ui.add(Button::new("Add a task")).clicked();
                if add_task_clicked {
                    let task = Task::new(Task::new_id(), String::new(), None, None, None);
                    self.editing_task = Some(task.id());
                    let previous = None;
                    self.state
                        .store
                        .with_transaction(|txn| txn.insert_task(previous, &task))
                        .expect("FIXME: handle error");
                }
                for task in self.state.list_tasks_for_display().iter() {
                    if self.editing_task == Some(task.id()) {
                        let mut title = task.title().to_string();
                        let response = ui.text_edit_singleline(&mut title);
                        if add_task_clicked {
                            // Focus the edited task's title when shown for the first time.
                            response.request_focus();
                        }
                        if response.changed() {
                            let mut task = task.clone();
                            task.set_title(title);
                            self.state
                                .store
                                .with_transaction(|txn| txn.put_task(&task))
                                .expect("FIXME: handle error");
                        }
                        if response.lost_focus() {
                            self.editing_task = None;
                            self.sift_save();
                        }
                    } else {
                        let checked = task.completed().is_some();
                        let mut checkbox_checked = checked;
                        ui.checkbox(&mut checkbox_checked, task.title());
                        if checkbox_checked != checked {
                            self.state.toggle_id(&task.id());
                            self.sift_save();
                        }
                    }
                }
            });
        });
    }
}
