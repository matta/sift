use std::path::Path;

use eframe::egui;

use sift_persist::MemoryStore;

pub struct App {
    store: MemoryStore,
}

impl App {
    pub fn load(path: &Path) -> anyhow::Result<App> {
        Ok(Self {
            store: MemoryStore::load(path)?,
        })
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut name = "Matt".to_string();
            ui.heading("My egui Application");
            ui.horizontal(|ui| {
                let name_label = ui.label("Your name: ");
                ui.text_edit_singleline(&mut name)
                    .labelled_by(name_label.id);
            });
            let mut age = 42;
            ui.add(egui::Slider::new(&mut age, 0..=120).text("age"));
            if ui.button("Increment").clicked() {
                age += 1;
            }
            ui.label(format!("Hello '{}', age {}", name, age));
            if age != 42 {
                println!("age now {age}");
            }

            ui.image(egui::include_image!("ferris.png"));
        });
    }
}
