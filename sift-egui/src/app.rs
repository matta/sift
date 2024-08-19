use std::path::Path;

use eframe::egui::{self, vec2, ScrollArea, TextStyle};

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
            let height = TextStyle::Body.resolve(ui.style()).size;
            ScrollArea::vertical().show_rows(ui, height, tasks.len(), |ui, row_range| {
                ui.allocate_space(vec2(ui.available_width(), 0.0));
                for i in row_range {
                    if let Some(value) = tasks.get(i) {
                        ui.label(value.title());
                    }
                }
            });

            // ui.horizontal(|ui| {
            //     let name_label = ui.label("Your name: ");
            //     ui.text_edit_singleline(&mut name)
            //         .labelled_by(name_label.id);
            // });
            // let mut age = 42;
            // ui.add(egui::Slider::new(&mut age, 0..=120).text("age"));
            // if ui.button("Increment").clicked() {
            //     age += 1;
            // }
            // ui.label(format!("Hello '{}', age {}", name, age));
            // if age != 42 {
            //     println!("age now {age}");
            // }

            // ui.image(egui::include_image!("ferris.png"));
        });
    }
}
