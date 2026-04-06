use std::path::PathBuf;

use egui::*;
use egui::containers::*;
use egui::widgets::*;

use crate::data::*;
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

    config_path: PathBuf,
    plugin_dir: PathBuf,
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

        let mut engine = TaskEngine::new();

        // Failure to load the config is a fatal error and continuing may cause data
        // loss, so we panic instead of just logging the error.
        if let Err(err) = engine.load_config(&config_path) {
            panic!("failed to load config at {}: {err:?}", config_path.display());
        }

        // Failure to scan plugins is not a fatal error: tasks that depend on missing
        // plugins will simply be invalid and won't run, but the user can still edit
        // the config and fix the problem. So we just log the error and continue.
        if let Err(err) = engine.scan_plugins(&plugin_dir) {
            log::error!("failed to scan plugins in {}: {err:?}", plugin_dir.display());
        }

        Self {
            engine,
            page: Page::default(),
            config_path,
            plugin_dir,
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
                self.ui_main(ui);
            });
    }
}

impl App {
    fn ui_nav(&mut self, ui: &mut Ui) {
        ui.selectable_value(&mut self.page, Page::Dashboard, "Dashboard");
        ui.selectable_value(&mut self.page, Page::Plugins, "Plugins");
        ui.separator();

    }

    fn ui_main(&mut self, ui: &mut egui::Ui) {
    }
}
