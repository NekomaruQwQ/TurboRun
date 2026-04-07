mod dashboard;
use dashboard::dashboard_ui;
mod plugin;
use plugin::plugins_ui;
mod task;
use task::task_ui;
mod edit_task;
use edit_task::edit_task_ui;

use crate::theme;
use crate::icon;
use crate::data::*;
use crate::engine::TaskEngine;

type PageResult = (
    Option<Action>,
    Option<Page>);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Page {
    Dashboard,
    Plugins,
    Task(TaskId),
    TaskEditor(Task),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Action {
    RunTask(TaskId),
    StopTask(TaskId),
    SaveTask(Task),
    DeleteTask(TaskId),
}

pub fn app_ui(
    ui: &mut egui::Ui,
    page: &mut Page,
    engine: &mut TaskEngine)
 -> Option<Action> {
    use egui::*;

    Panel::left("nav")
        .frame(
            Frame::new()
                .fill(theme::BG_NAV)
                .inner_margin(Margin::same(8)))
        .exact_size(180.0)
        .resizable(false)
        .show_inside(ui, |ui| {
            ui.with_layout(Layout::top_down_justified(Align::LEFT), |ui| {
                nav_ui(ui, page, engine);
            });
        });

    let InnerResponse {
        inner: (action, next_page),
        ..
    } = CentralPanel::default_margins()
        .show_inside(ui, |ui| {
            match *page {
                Page::Dashboard =>
                    dashboard_ui(ui, engine),
                Page::Plugins =>
                    plugins_ui(ui, engine),
                Page::Task(task_id) =>
                    task_ui(ui, engine, task_id),
                Page::TaskEditor(ref mut task) =>
                    edit_task_ui(ui, engine.plugins_sorted().collect::<Vec<_>>().as_slice(), task, engine.task(task.id).is_some()),
            }
        });

    if let Some(next_page) = next_page {
        *page = next_page;
    }

    action
}
fn nav_ui(
    ui: &mut egui::Ui,
    page: &mut Page,
    engine: &TaskEngine) {
    ui.selectable_value(page, Page::Dashboard, "Dashboard");
    ui.selectable_value(page, Page::Plugins, "Plugins");
    ui.separator();

    if ui.button(format!("{} New Task", icon::PLUS)).clicked() {
        // Brand-new task: fresh random id, empty fields, navigate to editor.
        // The editor decides "is_existing" by checking the engine for this id,
        // so a freshly-generated id naturally renders as "New Task".

        *page = Page::TaskEditor(engine.create_task());
    }

    for task in engine.tasks_sorted() {
        let task_id =
            task.task().id;
        let task_name =
            task.task().name.as_str();
        ui
            .button(task_name)
            .clicked()
            .then(|| *page = Page::Task(task_id));
    }
}