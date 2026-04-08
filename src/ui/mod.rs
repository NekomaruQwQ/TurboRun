mod common;

mod app;
pub use app::app_ui;
mod nav;
use nav::nav_ui;
mod dashboard;
use dashboard::dashboard_ui;
mod plugin;
use plugin::plugins_ui;
mod task_viewer;
use task_viewer::task_viewer_ui;
mod task_editor;
use task_editor::task_editor_ui;

use crate::icon;
use crate::data::*;
use crate::engine::TaskEngine;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Page {
    Dashboard,
    Plugins,
    TaskViewer(TaskId),
    TaskEditor(Task),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Action {
    RefreshPlugins,
    RunTask(TaskId),
    StopTask(TaskId),
    SaveTask(Task),
    DeleteTask(TaskId),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[derive(Default)]
pub struct ViewContext {
    action: Option<Action>,
    navigation: Option<Page>,
}

impl ViewContext {
    fn set_action(&mut self, action: Action) {
        self.action
            .is_some()
            .then(|| log::warn!("Overriding existing action {:?} with {:?}", self.action, action));
        self.action = Some(action);
    }

    fn set_navigation(&mut self, page: Page) {
        self.navigation
            .is_some()
            .then(|| log::warn!("Overriding existing navigation {:?} with {:?}", self.navigation, page));
        self.navigation = Some(page);
    }
}
