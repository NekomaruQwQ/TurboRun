use std::path::PathBuf;

use egui::*;

use crate::color;
use crate::ui;
use crate::data::*;
use crate::worker::TaskStatus;
use crate::engine::TaskEngine;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[derive(Default)]
enum Page {
    #[default] Dashboard,
    Plugins,
    Task(TaskId),
}

pub struct App {
    engine: TaskEngine,
    page: Page,
    task_edit: Option<Task>,
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
            page: Page::default(),
            task_edit: None,
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut Ui, _: &mut eframe::Frame) {
        Panel::left("nav")
            .min_size(180.0)
            .max_size(180.0)
            .resizable(false)
            .show_inside(ui, |ui| {
                ui.scope_builder(
                    UiBuilder::new()
                        .layout(Layout::top_down(Align::Min)
                        .with_cross_justify(true)),
                    |ui| self.ui_nav(ui))
            });
        CentralPanel::default_margins()
            .show_inside(ui, |ui| {
                match self.page {
                    Page::Dashboard =>
                        self.ui_dashboard(ui),
                    Page::Plugins =>
                        ui::plugins_ui(ui, &mut self.engine),
                    Page::Task(task_id) =>
                        self.ui_task(ui, task_id),
                }
            });
    }
}

impl App {
    fn ui_nav(&mut self, ui: &mut Ui) {
        ui.selectable_value(&mut self.page, Page::Dashboard, "Dashboard");
        ui.selectable_value(&mut self.page, Page::Plugins, "Plugins");
        ui.separator();
        for task in self.engine.tasks() {
            let task_id = task.task().id;
            let task_name = &task.task().name;
            let task_status = task.status(self.engine.plugins());
            ui
                .add_sized((ui.available_width(), 0.0), {
                    Button::selectable(self.page == Page::Task(task_id), "")
                        .left_text(task_name)
                        .right_text(format_task_status(&task_status))
                })
                .clicked()
                .then(|| self.page = Page::Task(task_id));
        }
    }

    fn ui_dashboard(&mut self, ui: &mut Ui) {
    }


    fn ui_task(&mut self, ui: &mut Ui, task_id: TaskId) {
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
