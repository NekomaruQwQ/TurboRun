pub use app::app_ui;
pub use style::setup_style;

mod color;
mod style;
mod widget;
mod common;

mod app;
mod nav;

mod page {
    use super::*;

    mod dashboard;
    mod plugin;
    mod task_viewer;
    mod task_editor;

    pub use dashboard::dashboard_ui;
    pub use plugin::plugin_ui;
    pub use task_viewer::task_viewer_ui;
    pub use task_editor::task_editor_ui;
}

const TASK_VIEWER_PLUGIN_CARD_COMPACT: bool = true;

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
    StartTask(TaskId),
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
