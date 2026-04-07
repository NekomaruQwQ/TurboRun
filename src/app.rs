use std::path::PathBuf;
use std::time::Duration;
use std::time::SystemTime;

use egui::*;
use tap::Pipe;

use crate::theme;
use crate::color;
use crate::icon;

use crate::data::*;
use crate::ui::PageResult;
use crate::worker::TaskStatus;
use crate::worker::TaskWorker;
use crate::engine::TaskEngine;

use crate::ui;
use crate::ui::*;

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
                    self.nav_ui(ui);
                });
            });
        CentralPanel::default_margins()
            .show_inside(ui, |ui| {
                match self.page {
                    Page::Dashboard =>
                        ui::dashboard_ui(ui, &mut self.engine),
                    Page::Plugins =>
                        ui::plugins_ui(ui, &mut self.engine),
                    Page::Task(task_id) =>
                        ui::task_ui(ui, &mut self.engine, task_id),
                    Page::TaskEditor(ref mut task) =>
                        ui::edit_task_ui(ui, &self.engine.plugins_sorted().collect::<Vec<_>>(), task, self.engine.task(task.id).is_some()),
                }.pipe(|res| self.handle_page_result(res));
            });
    }
}

impl App {

    fn handle_page_result(&mut self, (action, navigation): PageResult) {
        if let Some(action) = action {
            match action {
                PageAction::RunTask(id) => {
                    let task_mut =
                        self.engine
                            .tasks_mut()
                            .get_mut(&id)
                            .expect("task must be present to be run");
                    assert!(
                        !task_mut.is_running(),
                        "task must not be running to be started");
                    if let Err(err) = self.engine.run_task(id) {
                        log::error!("failed to start task {}: {err:?}", id);
                    }
                },
                PageAction::StopTask(id) => {
                    self.engine
                        .tasks_mut()
                        .get_mut(&id)
                        .expect("task must be present to be stopped")
                        .stop();
                },
                PageAction::SaveTask(task) => {
                    if let Some(worker) = self.engine.tasks_mut().get_mut(&task.id) {
                        // Update in place so the worker's `proc` / `last_result`
                        // survive — important if the user edits a running task.
                        *worker.task_mut() = task.clone();
                    } else {
                        self.engine
                            .tasks_mut()
                            .insert(task.id, TaskWorker::new(task.clone()));
                    }
                    if let Err(err) = self.engine.save_config() {
                        log::error!("save_config failed: {err:?}");
                    }
                },
                PageAction::DeleteTask(id) => {
                    if let Some(worker) = self.engine.task(id) {
                        if worker.is_running() {
                            log::error!("refusing to delete running task {}", id);
                        } else {
                            self.engine.tasks_mut().remove(&id);
                            if let Err(err) = self.engine.save_config() {
                                log::error!("save_config failed: {err:?}");
                            }
                        }
                    } else {
                        log::error!("failed to find task {} to delete", id);
                    }
                },
            }
        }

        match navigation {
            Some(PageNavigation::Dashboard) =>
                self.page = Page::Dashboard,
            Some(PageNavigation::Plugins) =>
                self.page = Page::Plugins,
            Some(PageNavigation::Task(id)) =>
                self.page = Page::Task(id),
            Some(PageNavigation::TaskEditor(id)) => {
                let task = self.engine.task(id)
                    .map(|worker| worker.task().clone())
                    .unwrap_or_else(|| Task {
                        id,
                        name: String::new(),
                        command: String::new(),
                        plugins: Vec::new(),
                        last_modified: SystemTime::now(),
                    });
                self.page = Page::TaskEditor(task);
            },
            Some(ui::PageNavigation::TaskEditerCreateNew) => {
                self.page = Page::TaskEditor(self.engine.create_task());
            },
            None => {},
        }
    }

    fn nav_ui(&mut self, ui: &mut Ui) {
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
