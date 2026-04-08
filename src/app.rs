use std::path::PathBuf;
use std::time::Duration;

use crate::engine::TaskEngine;
use crate::ui;

pub struct App {
    engine: TaskEngine,
    page: ui::Page,
}

impl eframe::App for App {
    fn logic(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        self.engine.update();

        // Drive live output at ~30fps regardless of user interaction.
        ctx.request_repaint_after(Duration::from_secs_f32(1.0 / 30.0));
    }

    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        if let Some(action) = ui::app_ui(ui, &mut self.page, &self.engine) {
            self.on_action(action);
        }
    }
}

impl App {
    pub fn new() -> Self {
        use clap::Parser as _;
        use crate::*;
        let args = Args::parse();
        let config_path = PathBuf::from(args.config_path);
        let plugin_dir = PathBuf::from(args.plugin_dir);
        let engine = TaskEngine::new(&config_path, &plugin_dir);
        Self {
            engine,
            page: ui::Page::Dashboard,
        }
    }

    fn on_action(&mut self, action: ui::Action) {
        match action {
            ui::Action::RefreshPlugins => {
                self.engine
                    .scan_plugins()
                    .unwrap_or_else(|err| {
                        log::error!(
                            "failed to scan plugins in {}: {err:?}",
                            self.engine.plugin_dir().display());
                    });
            },
            ui::Action::RunTask(id) => {
                if let Err(err) = self.engine.run_task(id) {
                    log::error!("failed to start task {id}: {err:?}");
                }
            },
            ui::Action::StopTask(id) => {
                self.engine.stop_task(id);
            },
            ui::Action::SaveTask(task) => {
                self.engine.update_or_insert_task(task);
                if let Err(err) = self.engine.save_config() {
                    log::error!("save_config failed: {err:?}");
                }
            },
            ui::Action::DeleteTask(id) => {
                if let Some(worker) = self.engine.task(id) {
                    if worker.is_running() {
                        log::error!("refusing to delete running task {id}");
                    } else {
                        self.engine.remove_task(id);
                        if let Err(err) = self.engine.save_config() {
                            log::error!("save_config failed: {err:?}");
                        }
                    }
                } else {
                    log::error!("failed to find task {id} to delete");
                }
            },
        }
    }
}
