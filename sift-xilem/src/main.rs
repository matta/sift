// Copyright 2024 the Xilem Authors
// SPDX-License-Identifier: Apache-2.0

// On Windows platform, don't show a console when opening the app.
#![windows_subsystem = "windows"]

use sift_core::save_name;
use sift_persist::{MemoryStore, Store as _, Task};
use sift_state::State;
use xilem::view::{button, checkbox, flex, textbox, Axis};
use xilem::{EventLoop, WidgetView, Xilem};

struct App {
    next_task: String,
    state: State,
}

impl App {
    fn add_task(&mut self) {
        if self.next_task.is_empty() {
            return;
        }
        // FIXME: make generating new tasks less cumbersome
        // FIXME: handle errors
        let task = Task::new(Task::new_id(), self.next_task.clone(), None, None, None);
        let previous = None;
        self.state
            .store
            .with_transaction(|txn| txn.insert_task(previous, &task))
            .expect("FIXME: handle error");
        self.next_task.clear();
        self.save();
    }

    fn save(&self) {
        self.state
            .store
            .save(&save_name())
            .expect("TODO: handle this error");
    }
}

fn app_logic(app: &mut App) -> impl WidgetView<App> {
    let input_box = textbox(app.next_task.clone(), |app: &mut App, new_value| {
        app.next_task = new_value;
    })
    .on_enter(|app: &mut App, _| {
        app.add_task();
    });
    let first_line = flex((
        input_box,
        button("Add task".to_string(), |app: &mut App| {
            app.add_task();
        }),
    ))
    .direction(Axis::Vertical);

    let tasks = app
        .state
        .list_tasks_for_display()
        .iter()
        .map(|task| {
            let id = task.id();
            let checkbox = checkbox(
                task.title(),
                task.is_completed(),
                move |app: &mut App, checked| {
                    if let Some(task) = app.state.get_task(&id) {
                        if checked != task.is_completed() {
                            app.state.toggle_id(&id);
                            app.save();
                        }
                    }
                },
            );
            let delete_button = button("Delete", move |app: &mut App| {
                app.state.delete_task(&id);
                app.save();
            });
            flex((checkbox, delete_button)).direction(Axis::Horizontal)
        })
        .collect::<Vec<_>>();

    flex((first_line, tasks))
}

fn main() {
    let app = App {
        next_task: String::default(),
        state: State::new(MemoryStore::load(&save_name()).unwrap()),
    };

    let app = Xilem::new(app, app_logic);
    app.run_windowed(EventLoop::with_user_event(), "First Example".into())
        .unwrap();
}
