use egui::*;
use egui_flex::*;

use crate::icon;
use crate::color;
use crate::engine::TaskEngine;
use crate::worker::TaskStatus;

use super::*;
use super::common::ActionButton;

pub fn dashboard_ui(
    flex: &mut FlexInstance,
    page: &mut ViewContext,
    engine: &TaskEngine) {
    for worker in engine.tasks_sorted() {
        let task_id = worker.task().id;
        flex.add_ui(item().grow(1.0), |ui| {
            task_card(
                ui,
                page,
                worker.task(),
                engine.task_status(task_id),
                engine.task_is_valid(task_id),
                worker.is_running());
        });
    }
}

fn task_card(
    ui: &mut Ui,
    page: &mut ViewContext,
    task: &Task,
    status: TaskStatus,
    is_valid: bool,
    is_running: bool) {
    let _ = common::card(ui, |ui| {
        Flex::horizontal()
            .w_full()
            .gap([4.0, 0.0].into())
            .align_items(FlexAlign::Center)
            .show(ui, |flex| {
                task_card_content(
                    flex,
                    page,
                    task,
                    status,
                    is_valid,
                    is_running);
            })
    });
}

fn task_card_content(
    flex: &mut FlexInstance,
    page: &mut ViewContext,
    task: &Task,
    status: TaskStatus,
    is_valid: bool,
    is_running: bool) {
    let status_ui =
        match status {
            TaskStatus::Invalid =>
                RichText::new("Invalid").color(color::ORANGE),
            TaskStatus::Stopped =>
                RichText::new("").weak(),
            TaskStatus::Running =>
                RichText::new("Running").color(color::BLUE),
            TaskStatus::Success =>
                RichText::new("Success").color(color::GREEN),
            TaskStatus::Failure =>
                RichText::new("Failed").color(color::RED),
        }.small();

    // Run — disabled while already running or invalid.
    flex.add(
            item(),
            ActionButton::new()
                .icon(icon::PLAY)
                .tooltip("Run Task")
                .enabled(!is_running && is_valid))
        .on_hover_cursor(CursorIcon::PointingHand)
        .clicked()
        .then(|| page.set_action(Action::RunTask(task.id)));

    // Stop — disabled when not running.
    flex.add(
            item(),
            ActionButton::new()
                .icon(icon::STOP)
                .tooltip("Stop Task")
                .enabled(is_running))
        .on_hover_cursor(CursorIcon::PointingHand)
        .clicked()
        .then(|| page.set_action(Action::StopTask(task.id)));

    // Task name + status. Clicking anywhere on this opens the task viewer.
    flex
        .add(
            item().grow(1.0),
            Button::new("")
                .left_text(&task.name)
                .right_text(status_ui)
                .truncate())
        .on_hover_cursor(CursorIcon::PointingHand)
        .clicked()
        .then(|| page.set_navigation(Page::TaskViewer(task.id)));

    // Edit — disabled when running.
    flex
        .add(item(),
            ActionButton::new()
                .icon(icon::EDIT)
                .tooltip("Edit")
                .enabled(!is_running))
        .on_hover_cursor(CursorIcon::PointingHand)
        .clicked()
        .then(|| page.set_navigation(Page::TaskEditor(task.clone())));
}
