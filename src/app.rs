use std::path::PathBuf;
use std::time::Duration;
use std::time::SystemTime;

use egui::*;

use crate::theme;
use crate::color;
use crate::icon;

use crate::data::*;
use crate::worker::TaskStatus;
use crate::worker::TaskWorker;
use crate::engine::TaskEngine;

use crate::ui;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Page {
    Dashboard,
    Plugins,
    Task(TaskId),
    TaskEditor(Task),
}

pub struct App {
    engine: TaskEngine,
    page: Page,
}

impl App {
    pub fn new() -> Self {
        use clap::Parser as _;
        use crate::*;
        let args = Args::parse();

        let config_path =
            PathBuf::from(args.config_path);
        let plugin_dir =
            PathBuf::from(args.plugin_dir);

        let mut engine =
            TaskEngine::new(
                config_path.clone(),
                plugin_dir.clone());

        // Failure to load the config is a fatal error and continuing may cause data
        // loss, so we panic instead of just logging the error.
        if let Err(err) = engine.load_config() {
            panic!("failed to load config at {}: {err:?}", config_path.display());
        }

        // Failure to scan plugins is not a fatal error: tasks that depend on missing
        // plugins will simply be invalid and won't run, but the user can still edit
        // the config and fix the problem. So we just log the error and continue.
        if let Err(err) = engine.scan_plugins() {
            log::error!("failed to scan plugins in {}: {err:?}", plugin_dir.display());
        }

        Self {
            engine,
            page: Page::Dashboard,
        }
    }
}

impl eframe::App for App {
    fn logic(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        self.engine.update();

        // Drive live output at ~30fps regardless of user interaction.
        ctx.request_repaint_after(Duration::from_secs_f32(1.0 / 30.0));
    }

    fn ui(&mut self, ui: &mut Ui, _: &mut eframe::Frame) {
        Panel::left("nav")
            .frame(
                Frame::new()
                    .fill(theme::BG_NAV)
                    .inner_margin(Margin::same(8)))
            .exact_size(180.0)
            .resizable(false)
            .show_inside(ui, |ui| {
                ui.with_layout(Layout::top_down_justified(Align::LEFT), |ui| {
                    self.ui_nav(ui);
                });
            });
        CentralPanel::default_margins()
            .show_inside(ui, |ui| {
                match self.page {
                    Page::Dashboard =>
                        self.ui_dashboard(ui),
                    Page::Plugins =>
                        ui::plugins_ui(ui, &mut self.engine),
                    Page::Task(task_id) =>
                        ui::task_ui(ui, &mut self.engine, task_id),
                    Page::TaskEditor(task) =>
                        self.ui_edit_task(ui, task),
                }
            });
    }
}

impl App {
}

impl App {
    fn ui_nav(&mut self, ui: &mut Ui) {
        ui.selectable_value(&mut self.page, Page::Dashboard, "Dashboard");
        ui.selectable_value(&mut self.page, Page::Plugins, "Plugins");
        ui.separator();

        if ui.button(format!("{} New Task", icon::PLUS)).clicked() {
            // Brand-new task: fresh random id, empty fields, navigate to editor.
            // The editor decides "is_existing" by checking the engine for this id,
            // so a freshly-generated id naturally renders as "New Task".

            self.page = Page::TaskEditor(self.engine.create_task());
        }

        for task in self.engine.tasks() {
            let task_id =
                task.task().id;
            let task_name =
                task.task().name.as_str();
            let task_status =
                task.status(self.engine.plugins());
            ui
                .button(task_name)
                .clicked()
                .then(|| self.page = Page::Task(task_id));
        }
    }

    fn ui_dashboard(&mut self, ui: &mut Ui) {
        // The dashboard page itself lives in `ui::dashboard`. This wrapper just
        // owns the App-side reaction to navigation actions (opening the editor),
        // since `dashboard_ui` only sees `&mut TaskEngine` and can't touch
        // `self.task_edit` / `self.page`.
        match ui::dashboard_ui(ui, &mut self.engine) {
            ui::DashboardAction::None => {}
            ui::DashboardAction::EditTask(id) => {
                // Open the editor with a clone of the existing task. The clone
                // isolates edits from the live worker until Save commits them.
                if let Some(worker) = self.engine.task(id) {
                    self.task_edit = Some(ui::TaskEditor::new(worker.task().clone()));
                    self.page = Page::TaskEditor;
                }
            }
            ui::DashboardAction::NewTask => {
                // Brand-new task: fresh random id, empty fields, navigate to editor.
                // The editor decides "is_existing" by checking the engine for this id,
                // so a freshly-generated id naturally renders as "New Task".
                self.task_edit = Some(ui::TaskEditor::new(Task {
                    id: TaskId::random(),
                    name: String::new(),
                    command: String::new(),
                    plugins: Vec::new(),
                    last_modified: SystemTime::now(),
                }));
                self.page = Page::TaskEditor;
            }
        }
    }

    fn ui_edit_task(&mut self, ui: &mut Ui) {
        // Phase 1: render. Scope the editor borrow so it ends before Phase 2
        // touches `self.engine` / `self.task_edit` mutably.
        let action = {
            let Some(editor) = self.task_edit.as_mut() else {
                // Defensive fallback for the impossible case where we navigated
                // to EditTask without a draft. Don't panic — give the user a
                // way back.
                ui.label("(no task being edited)");
                if ui.button("Back").clicked() {
                    self.page = Page::Dashboard;
                }
                return;
            };
            // Disjoint borrow: `&self.engine` and `&mut self.task_edit` are
            // independent fields of `self`, so this compiles.
            let is_existing = self.engine.task(editor.task.id).is_some();
            ui::edit_task_ui(ui, &self.engine, editor, is_existing)
        };

        // Phase 2: apply the action.
        match action {
            ui::EditAction::None => {}
            ui::EditAction::Save => {
                // `editor.finalize()` already ran inside the editor function,
                // so `task_edit.task.plugins[i].vars` reflects the row state.
                if let Some(editor) = self.task_edit.take() {
                    let id = editor.task.id;
                    if let Some(worker) = self.engine.tasks_mut().get_mut(&id) {
                        // Update in place so the worker's `proc` / `last_result`
                        // survive — important if the user edits a running task.
                        *worker.task_mut() = editor.task;
                    } else {
                        self.engine
                            .tasks_mut()
                            .insert(id, TaskWorker::new(editor.task));
                    }
                    if let Err(err) = self.engine.save_config() {
                        log::error!("save_config failed: {err:?}");
                    }
                    self.page = Page::Dashboard;
                }
            }
            ui::EditAction::Cancel => {
                self.task_edit = None;
                self.page = Page::Dashboard;
            }
            ui::EditAction::Delete => {
                if let Some(editor) = self.task_edit.take() {
                    let id = editor.task.id;
                    // Stop first so we don't leak a child process when the
                    // worker is removed from the map.
                    self.engine.stop_task(id);
                    self.engine.tasks_mut().remove(&id);
                    if let Err(err) = self.engine.save_config() {
                        log::error!("save_config failed: {err:?}");
                    }
                    self.page = Page::Dashboard;
                }
            }
        }
    }
}

fn format_task_status(status: &TaskStatus) -> RichText {
    match *status {
        TaskStatus::Invalid =>
            RichText::new("Invalid").color(color::ORANGE),
        TaskStatus::Stopped =>
            RichText::new("").weak(),
        TaskStatus::Running =>
            RichText::new("Running").color(color::BLUE),
        TaskStatus::Success(_) =>
            RichText::new("Success").color(color::GREEN),
        TaskStatus::Failure(_) =>
            RichText::new("Failed").color(color::RED),
    }.small()
}
