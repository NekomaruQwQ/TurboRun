use egui::*;
use egui_flex::*;

use crate::color;
use crate::engine::TaskEngine;

use super::*;

pub fn app_ui(
    ui: &mut egui::Ui,
    page: &mut Page,
    engine: &TaskEngine)
 -> Option<Action> {
    let mut view = ViewContext::default();

    Panel::left("nav")
        .default_size(200.0)
        .resizable(false)
        .frame(
            Frame::new()
                .fill(color::BACKGROUND_ALT)
                .inner_margin(Margin::same(8)))
        .show_inside(ui, |ui| {
            Flex::vertical()
                .w_full()
                .gap(Vec2::new(4.0, 4.0))
                .show(ui, |flex| nav_ui(flex, &mut view, page, engine))
        });

    CentralPanel::default()
        .show_inside(ui, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                match *page {
                    Page::Dashboard =>
                        Flex::vertical()
                            .w_full()
                            .gap([8.0, 8.0].into())
                            .show(ui, |flex| dashboard_ui(flex, &mut view, engine))
                            .inner,
                    Page::Plugins =>
                        plugins_ui(ui, &mut view, engine),
                    Page::TaskViewer(task_id) =>
                        task_viewer_ui(ui, &mut view, engine, task_id),
                    Page::TaskEditor(ref mut task) =>
                        task_editor_ui(
                            ui,
                            &mut view,
                            engine.plugins_sorted().collect::<Vec<_>>().as_slice(),
                            task,
                            engine.task(task.id).is_some()),
                }
            })
        });

    if let Some(navigation) = view.navigation {
        *page = navigation;
    }

    view.action
}
