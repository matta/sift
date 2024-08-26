// Copyright 2024 the Xilem Authors
// SPDX-License-Identifier: Apache-2.0

// On Windows platform, don't show a console when opening the app.
#![windows_subsystem = "windows"]

use sift_core::save_name;
use sift_persist::{MemoryStore, Store, Task, TaskId};
use sift_state::State;
use xilem::view::{button, checkbox, flex, label, textbox, Axis, CrossAxisAlignment};
use xilem::{EventLoop, WidgetView, Xilem};

enum Screen {
    Main,
    Edit { id: TaskId },
}

struct App {
    screen: Screen,
    state: State,
}

impl App {
    fn add_task(&mut self) {
        let title = String::default();
        let id = Task::new_id();
        let task = Task::new(id, title, None, None, None);
        let previous = None;
        self.state
            .store
            .with_transaction(|txn| txn.insert_task(previous, &task))
            .expect("FIXME: handle error");
        self.screen = Screen::Edit { id };
    }

    fn save(&self) {
        self.state
            .store
            .save(&save_name())
            .expect("TODO: handle this error");
    }
}

fn app_logic(app: &mut App) -> impl WidgetView<App> {
    match &app.screen {
        Screen::Main => main_app_logic(app).boxed(),
        Screen::Edit { id } => edit_app_logic(*id, app).boxed(),
    }
}

fn main_app_logic(app: &mut App) -> impl WidgetView<App> {
    let add_task = button("Add task", |app: &mut App| {
        app.add_task();
    });

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

    flex((
        add_task,
        flex(tasks)
            //  Align rows to the left
            .cross_axis_alignment(CrossAxisAlignment::Start),
    ))
}

fn edit_app_logic(id: TaskId, app: &mut App) -> impl WidgetView<App> {
    let task = app
        .state
        .store
        .get_task(&id)
        .expect("FIXME: task must exist");
    let title = task.title().to_string();
    let label = label("Edit task");
    let input_box = textbox(title, move |app: &mut App, new_value| {
        let mut task = app.state.store.get_task(&id).expect("task exists");
        app.state
            .store
            .with_transaction(move |txn| {
                task.set_title(new_value);
                txn.put_task(&task)
            })
            .expect("FIXME: handle error");
    })
    .on_enter(|app: &mut App, _| {
        app.save();
        app.screen = Screen::Main;
    });
    flex((label, input_box)).direction(Axis::Vertical)
}

fn main() {
    let app = App {
        screen: Screen::Main,
        state: State::new(MemoryStore::load(&save_name()).unwrap()),
    };

    let app = Xilem::new(app, app_logic);
    app.run_windowed(EventLoop::with_user_event(), "First Example".into())
        .unwrap();
}
