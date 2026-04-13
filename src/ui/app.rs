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
        use egui::scroll_area::ScrollBarVisibility as ScrollBar;
        ScrollArea::vertical()
            .scroll_bar_visibility(ScrollBar::AlwaysHidden)
            .show(ui, |ui| {
                Flex::vertical()
                    .w_full()
                    .gap([8.0, 8.0].into())
                    .show(ui, |flex| match *page {
                        Page::Dashboard =>
                            page::dashboard_ui(flex, &mut view, engine),
                        Page::Plugins =>
                            page::plugin_ui(flex, &mut view, engine),
                        Page::TaskViewer(task_id) =>
                            page::task_viewer_ui(flex, &mut view, engine, task_id),
                        Page::TaskEditor(ref mut task) =>
                            page::task_editor_ui(flex, &mut view, engine, task),
                    });
            })
    });

    if let Some(navigation) = view.navigation {
        *page = navigation;
    }

    view.action
}
