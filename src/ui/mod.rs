mod plugin;
pub use plugin::plugins_ui;

mod task;
pub use task::task_ui;

mod edit_task;
pub use edit_task::edit_task_ui;

mod dashboard;
pub use dashboard::dashboard_ui;

use crate::data::*;

pub type PageResult = (
    Option<PageAction>,
    Option<PageNavigation>);

pub enum PageAction {
    RunTask(TaskId),
    StopTask(TaskId),
    SaveTask(Task),
    DeleteTask(TaskId),
}

pub enum PageNavigation {
    Dashboard,
    Plugins,
    Task(TaskId),
    TaskEditor(TaskId),
    TaskEditerCreateNew,
}
