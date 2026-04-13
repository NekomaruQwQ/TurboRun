use std::path::Path;
use std::time::Duration;

use crate::engine::*;
use crate::ui;
use crate::Args;

pub struct App {
    engine: TaskEngine,
    page: ui::Page,
    args: Args,
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
        log::info!("starting with config: {}, plugin packs: {:?}",
            args.config,
            args.plugin_pack);

        let mut engine = TaskEngine::default();

        // Failure to load the config is a fatal error and continuing may cause data
        // loss, so we panic instead of just logging the error.
        engine
            .load_config(Path::new(&args.config))
            .expect("failed to load config file");

        // Failure to scan plugins is not a fatal error: tasks that depend on missing
        // plugins will simply be invalid and won't run, but the user can still edit
        // the config and fix the problem. So we just log the error and continue.
        engine
            .load_plugin_packs(
                args.plugin_pack.iter().map(Path::new));
        Self {
            engine,
            page: ui::Page::Dashboard,
            args,
        }
    }

    fn on_action(&mut self, action: ui::Action) {
        match action {
            ui::Action::RefreshPlugins => {
                self.engine.load_plugin_packs(
                    self.args.plugin_pack.iter().map(Path::new));
            },
            ui::Action::StartTask(id) => {
                if let Err(err) = self.engine.start_task(id) {
                    log::error!("failed to start task {id}: {err:?}");
                }
            },
            ui::Action::StopTask(id) => {
                if let Err(err) = self.engine.stop_task(id) {
                    log::error!("failed to stop task {id}: {err:?}");
                }
            },
            ui::Action::SaveTask(task) => {
                self.engine.update_or_insert_task(task);
                if let Err(err) = self.engine.save_config(Path::new(&self.args.config)) {
                    log::error!("save_config failed: {err:?}");
                }
            },
            ui::Action::DeleteTask(id) => {
                if self.engine.task(id).is_some() {
                    if self.engine.task_status(id) == TaskStatus::Running {
                        log::error!("refusing to delete running task {id}");
                    } else {
                        self.engine.remove_task(id);
                        if let Err(err) = self.engine.save_config(Path::new(&self.args.config)) {
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
