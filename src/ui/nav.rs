use tap::prelude::*;
use egui::*;
use egui_flex::*;

use super::*;
use super::widget::*;

pub fn nav_ui(
    flex: &mut FlexInstance,
    view: &mut ViewContext,
    page: &Page,
    engine: &TaskEngine) {
    flex.add(
        item(),
        Button::new("")
            .left_text(format!("{}  Dashboard", nf::md::MD_VIEW_DASHBOARD))
            .selected(matches!(page, Page::Dashboard)))
        .on_hover_cursor(CursorIcon::PointingHand)
        .clicked()
        .then(|| view.set_navigation(Page::Dashboard));
    flex.add_flex(
        item(),
        Flex::horizontal()
            .w_full()
            .gap([4.0, 4.0].into()),
        |flex| {
            let plugin_count =
                engine
                    .plugin_packs()
                    .values()
                    .map(|pack| pack.plugins.len())
                    .sum::<usize>();
            flex.add(
                item().grow(1.0),
                Button::new("")
                    .left_text(format!("{}  Plugins", nf::fa::FA_PUZZLE_PIECE))
                    .right_text(
                        RichText::new(format!("{plugin_count} loaded"))
                            .small()
                            .weak())
                    .selected(matches!(page, Page::Plugins)))
                .on_hover_cursor(CursorIcon::PointingHand)
                .clicked()
                .then(|| view.set_navigation(Page::Plugins));
            flex.add(item(), FlexActionButton::new().icon(nf::fa::FA_ARROWS_ROTATE))
                .on_hover_cursor(CursorIcon::PointingHand)
                .clicked()
                .then(|| view.set_action(Action::RefreshPlugins));
        });

    flex.add(item(), FlexSeparator);

    let is_editing_new_task =
        matches!(page, Page::TaskEditor(t) if engine.task(t.id).is_none());
    flex.add(
        item(),
        Button::new("")
            .left_text(format!("{}  New Task", nf::fa::FA_PLUS))
            .selected(is_editing_new_task))
        .on_hover_cursor(CursorIcon::PointingHand)
        .clicked()
        .then(|| view.set_navigation(Page::TaskEditor(engine.empty_task())));

    flex.add_ui(item().grow(1.0), |ui| {
        ScrollArea::vertical().show(ui, |ui| {
            Flex::vertical()
                .w_full()
                .gap([4.0, 4.0].into())
                .show(ui, |flex| {
                    for worker in engine.tasks_sorted() {
                        nav_task_ui(
                            flex,
                            view,
                            page,
                            worker.task());
                    }
                });
        });
    });
}

fn nav_task_ui(
    flex: &mut FlexInstance,
    view: &mut ViewContext,
    page: &Page,
    task: &Task) {
    let task_selected =
        matches!(page, Page::TaskViewer(id) if *id == task.id) |
        matches!(page, Page::TaskEditor(t) if t.id == task.id);
    let task_editor_selected =
        matches!(page, Page::TaskEditor(t) if t.id == task.id);
    flex.add_flex(
        item(),
        Flex::horizontal()
            .id_salt(format!("nav_task_{}", task.id))
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
                .then(|| view.set_navigation(Page::TaskViewer(task.id)));
            flex.add(
                item(),
                FlexActionButton::new()
                    .icon(nf::fa::FA_PEN)
                    .selected(task_editor_selected))
                .on_hover_cursor(CursorIcon::PointingHand)
                .clicked()
                .pipe(|clicked| clicked && !task_editor_selected)
                .then(|| view.set_navigation(Page::TaskEditor(task.clone())));
        });
}