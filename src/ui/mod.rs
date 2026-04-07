mod widget;

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

fn nav_ui(
    flex: &mut egui_flex::FlexInstance,
    page: &Page,
    engine: &TaskEngine)
 -> PageResult {
    use egui::*;
    use egui_flex::*;

    let mut action = None;
    let mut next_page = None;

    flex.add(
        item(),
        Button::new("")
            .left_text(format!("{} Dashboard", ""))
            .selected(page == &Page::Dashboard))
        .clicked()
        .then(|| next_page = Some(Page::Dashboard));
    flex.add_flex(
        item(),
        Flex::horizontal()
            .w_full()
            .gap([4.0, 4.0].into()),
        |flex| {
            flex.add(
                item().grow(1.0),
                Button::new("")
                    .left_text(format!("{} Plugins", ""))
                    .selected(page == &Page::Plugins))
                .clicked()
                .then(|| next_page = Some(Page::Plugins));
            flex.add_ui(item(), |ui| {
                widget::action_button(ui, true, icon::REFRESH, "Refresh Plugins")
                    .clicked()
                    .then(|| action = Some(Action::RefreshPlugins));
            });
        });

    // This is a more "polite" implementation of a separator that
    // does not try to consume all available width (which would
    // cause unwanted stretching of the side panel and its contents).
    flex.add_ui(item(), |ui| {
        let (rect, _) =
            ui.allocate_exact_size(
                Vec2::new(ui.available_width(), 1.0),
                Sense::hover());
        ui.painter().hline(
            rect.x_range(),
            rect.center().y,
            Stroke::new(1.0, theme::COLOR_BORDER));
    });

    flex.add(
        item(),
        Button::new("")
            .left_text(format!("{}  New Task", icon::PLUS)))
        .clicked()
        .then(|| next_page = Some(Page::TaskEditor(engine.empty_task())));

    for worker in engine.tasks_sorted() {
        let task = worker.task();

        flex.add_flex(
            item(),
            Flex::horizontal()
                .w_full()
                .gap([4.0, 4.0].into()),
            |flex| {
                flex.add(
                    item().grow(1.0),
                    Button::new("")
                        .left_text(&task.name)
                        .selected(page == &Page::TaskViewer(task.id)))
                    .clicked()
                    .then(|| next_page = Some(Page::TaskViewer(task.id)));
                flex.add_ui(
                    item(),
                    |ui| {
                        widget::action_button(ui, true, icon::PENCIL, "Edit Task")
                            .clicked()
                            .then(|| next_page = Some(Page::TaskEditor(task.clone())));
                });
            });
    }

    (action, next_page)
}
