use egui::*;
use egui_flex::*;

use super::color;
use crate::engine::TaskEngine;
use crate::worker::TaskStatus;

use super::*;
use super::widget::FlexActionButton;
use super::common::task_status_label;

pub fn dashboard_ui(
    flex: &mut FlexInstance,
    view: &mut ViewContext,
    engine: &TaskEngine) {
    for worker in engine.tasks_sorted() {
        flex.add_ui(item(), |ui| {
            task_card(
                ui,
                view,
                worker.task(),
                engine.task_status(worker.task().id));
        });
    }
}

fn task_card(
    ui: &mut Ui,
    view: &mut ViewContext,
    task: &Task,
    status: TaskStatus) {
    Frame::new()
        .fill(color::CARD)
        .corner_radius(6.0)
        .inner_margin(Margin::same(4))
        .show(ui, |ui| {
            Flex::horizontal()
                .id_salt(format!("dashboard_task_{}", task.id))
                .w_full()
                .gap([4.0, 0.0].into())
                .show(ui, |flex| {
                    task_card_content(
                        flex,
                        view,
                        task,
                        status);
                })
        });
}

fn task_card_content(
    flex: &mut FlexInstance,
    view: &mut ViewContext,
    task: &Task,
    status: TaskStatus) {
    // Run — disabled while already running or invalid.
    flex.add(
            item(),
            FlexActionButton::new()
                .icon(nf::fa::FA_PLAY)
                .enabled(status.can_start()))
        .on_hover_cursor(CursorIcon::PointingHand)
        .clicked()
        .then(|| view.set_action(Action::RunTask(task.id)));

    // Stop — disabled when not running.
    flex.add(
            item(),
            FlexActionButton::new()
                .icon(nf::fa::FA_STOP)
                .enabled(status.can_stop()))
        .on_hover_cursor(CursorIcon::PointingHand)
        .clicked()
        .then(|| view.set_action(Action::StopTask(task.id)));

    // Task name + status. Clicking anywhere on this opens the task viewer.
    flex
        .add(
            item().grow(1.0),
            Button::new("")
                .left_text(task.name.as_str())
                .right_text(task_status_label(status).small())
                .truncate())
        .on_hover_cursor(CursorIcon::PointingHand)
        .clicked()
        .then(|| view.set_navigation(Page::TaskViewer(task.id)));

    // Edit — disabled when running.
    flex
        .add(item(),
            FlexActionButton::new()
                .icon(nf::fa::FA_PEN)
                .enabled(status.can_edit()))
        .on_hover_cursor(CursorIcon::PointingHand)
        .clicked()
        .then(|| view.set_navigation(Page::TaskEditor(task.clone())));

    // let delete_confirm_id = Id::new(("dashboard.delete_confirm", task.id));
    // let delete_confirm =
    //     flex.ui()
    //         .data_mut(|data| data.get_temp::<bool>(delete_confirm_id))
    //         .unwrap_or(false);
}
