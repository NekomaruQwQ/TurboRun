use smol_str::{SmolStr, format_smolstr};
use tap::prelude::*;

use egui::*;
use egui_flex::*;

use crate::engine::TaskEngine;
use super::color;

use super::*;

pub fn app_ui(ui: &mut egui::Ui, page: &mut Page, engine: &TaskEngine)
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
                .show(ui, |flex| nav::nav_ui(flex, &mut view, page, engine))
        });

    CentralPanel::default().show_inside(ui, |ui| {
        ScrollArea::vertical().show(ui, |ui| {
            page
                .clone()
                .pipe(|page| match page {
                    Page::Dashboard =>
                        SmolStr::new_static("dashboard"),
                    Page::Plugins =>
                        SmolStr::new_static("plugins"),
                    Page::TaskViewer(task_id) =>
                        format_smolstr!("task_viewer_{task_id}"),
                    Page::TaskEditor(task) =>
                        format_smolstr!("task_editor_{}", task.id),
                })
                .pipe(|hash| ui.push_id(hash, |ui| match *page {
                    Page::Dashboard =>
                        Flex::vertical()
                            .w_full()
                            .gap([8.0, 8.0].into())
                            .show(ui, |flex| page::dashboard_ui(flex, &mut view, engine))
                            .inner,
                    Page::Plugins =>
                        page::plugin_ui(ui, &mut view, engine),
                    Page::TaskViewer(task_id) =>
                        Flex::vertical()
                            .w_full()
                            .gap([8.0, 8.0].into())
                            .show(ui, |flex| page::task_viewer_ui(flex, &mut view, engine, task_id))
                            .inner,
                    Page::TaskEditor(ref mut task) =>
                        page::task_editor_ui(
                            ui,
                            &mut view,
                            engine.plugin_packs(),
                            task,
                            engine.task(task.id).is_some()),
                }));
        })
    });

    if let Some(navigation) = view.navigation {
        *page = navigation;
    }

    view.action
}
