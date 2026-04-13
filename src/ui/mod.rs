pub use app::app_ui;
pub use style::setup_style;

mod prelude {
    pub use tap::prelude::*;

    pub use egui::*;
    pub use egui_flex::*;

    pub use crate::data::*;
    pub use crate::engine::*;

    pub use super::*;
    pub use super::widget::*;
}

mod color;
mod style;
mod widget;
mod custom;

mod app;
mod nav;

mod page {
    use super::prelude;

    mod dashboard;
    mod plugin;
    mod task_viewer;
    mod task_editor;

    pub use dashboard::dashboard_ui;
    pub use plugin::plugin_ui;
    pub use task_viewer::task_viewer_ui;
    pub use task_editor::task_editor_ui;
}

use crate::data::Task;
use crate::data::TaskId;

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
