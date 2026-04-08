use tap::prelude::*;
use egui::*;
use egui_flex::*;

use super::*;
use super::common::ActionButton;

pub fn nav_ui(
    flex: &mut egui_flex::FlexInstance,
    page: &Page,
    engine: &TaskEngine)
 -> PageResult {
    let mut action = None;
    let mut next_page = None;

    flex.add(
        item(),
        Button::new("")
            .left_text(format!("{} Dashboard", ""))
            .selected(page == &Page::Dashboard))
        .on_hover_cursor(CursorIcon::PointingHand)
        .clicked()
        .then(|| next_page = Some(Page::Dashboard));
    flex.add_flex(
        item(),
        Flex::horizontal()
            .w_full()
            .gap([4.0, 4.0].into()),
        |flex| {
            let plugin_count = engine.plugins_sorted().len();
            flex.add(
                item().grow(1.0),
                Button::new("")
                    .left_text(format!("{} Plugins", ""))
                    .right_text(
                        RichText::new(format!("{plugin_count} loaded"))
                            .small()
                            .weak())
                    .selected(page == &Page::Plugins))
                .on_hover_cursor(CursorIcon::PointingHand)
                .clicked()
                .then(|| next_page = Some(Page::Plugins));
            flex.add(
                item(),
                ActionButton::new()
                    .icon(icon::REFRESH)
                    .tooltip("View Plugins"))
                .on_hover_cursor(CursorIcon::PointingHand)
                .clicked()
                .then(|| action = Some(Action::RefreshPlugins));
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

    let is_editing_new_task =
        matches!(page, Page::TaskEditor(t) if engine.task(t.id).is_none());
    flex.add(
        item(),
        Button::new("")
            .left_text(format!("{}  New Task", icon::PLUS))
            .selected(is_editing_new_task))
        .on_hover_cursor(CursorIcon::PointingHand)
        .clicked()
        .then(|| next_page = Some(Page::TaskEditor(engine.empty_task())));

    flex.add_ui(item().grow(1.0), |ui| {
        ScrollArea::vertical().show(ui, |ui| {
            Flex::vertical()
                .w_full()
                .gap([4.0, 4.0].into())
                .show(ui, |flex| {
                    for worker in engine.tasks_sorted() {
                        nav_task_ui(
                            flex,
                            page,
                            &mut next_page,
                            worker.task());
                    }
                });
        });
    });

    (action, next_page)
}

fn nav_task_ui(
    flex: &mut FlexInstance,
    page: &Page,
    next_page: &mut Option<Page>,
    task: &Task) {
    let task_selected =
        matches!(page, Page::TaskViewer(id) if *id == task.id) |
        matches!(page, Page::TaskEditor(t) if t.id == task.id);
    let task_editor_selected =
        matches!(page, Page::TaskEditor(t) if t.id == task.id);
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
                    .selected(task_selected))
                .on_hover_cursor(CursorIcon::PointingHand)
                .clicked()
                .then(|| *next_page = Some(Page::TaskViewer(task.id)));
            flex.add(
                item(),
                ActionButton::new()
                    .icon(icon::PENCIL)
                    .tooltip("Edit Task")
                    .selected(task_editor_selected))
                .on_hover_cursor(CursorIcon::PointingHand)
                .clicked()
                .pipe(|clicked| clicked && !task_editor_selected)
                .then(|| *next_page = Some(Page::TaskEditor(task.clone())));
        });
}
