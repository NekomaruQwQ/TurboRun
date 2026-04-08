mod common;

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

pub fn app_ui(
    ui: &mut egui::Ui,
    page: &mut Page,
    engine: &mut TaskEngine)
 -> Option<Action> {
    use tap::prelude::*;
    use egui::*;
    use egui_flex::*;

    let mut final_action = None;

    Panel::left("nav")
        .default_size(200.0)
        .resizable(false)
        .frame(
            Frame::new()
                .fill(theme::COLOR_BACKGROUND_ALT)
                .inner_margin(Margin::same(8)))
        .show_inside(ui, |ui| {
            Flex::vertical()
                .w_full()
                .gap(Vec2::new(4.0, 4.0))
                .show(ui, |flex| nav_ui(flex, page, engine))
        })
        .inner
        .inner
        .pipe(|(action, next_page)| {
            if let Some(next_page) = next_page {
                *page = next_page;
            }

            if let Some(action) = action {
                final_action = Some(action);
            }
        });

    CentralPanel::default()
        .show_inside(ui, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                match *page {
                    Page::Dashboard =>
                        Flex::vertical()
                            .w_full()
                            .gap([8.0, 8.0].into())
                            .show(ui, |flex| dashboard_ui(flex, engine))
                            .inner,
                    Page::Plugins =>
                        plugins_ui(ui, engine),
                    Page::TaskViewer(task_id) =>
                        task_viewer_ui(ui, engine, task_id),
                    Page::TaskEditor(ref mut task) =>
                        task_editor_ui(
                            ui,
                            engine.plugins_sorted().collect::<Vec<_>>().as_slice(),
                            task,
                            engine.task(task.id).is_some()),
                }
            })
        })
        .inner
        .inner
        .pipe(|(action, next_page)| {
            if let Some(next_page) = next_page {
                *page = next_page;
            }

            if let Some(action) = action {
                final_action = Some(action);
            }
        });

    final_action
}
