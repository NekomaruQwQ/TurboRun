use egui::*;
use egui_flex::*;

use crate::color;
use crate::icon;
use crate::engine::TaskEngine;
use crate::worker::TaskStatus;

use super::*;

pub fn dashboard_ui(flex: &mut FlexInstance, engine: &TaskEngine) -> PageResult {
    let mut outer_action = None;
    let mut outer_next_page = None;

    for worker in engine.tasks_sorted() {
        let task_id = worker.task().id;

        flex.add_ui(item().grow(1.0), |ui| {
            let (action, next_page) =
                task_card(
                    ui,
                    worker.task(),
                    engine.task_status(task_id),
                    worker.is_running(),
                    engine.task_is_valid(task_id));
            if let Some(action) = action {
                outer_action = Some(action);
            }
            if let Some(next_page) = next_page {
                outer_next_page = Some(next_page);
            }
        });
    }

    (outer_action, outer_next_page)
}

fn task_card(
    ui: &mut Ui,
    task: &Task,
    status: TaskStatus,
    is_running: bool,
    is_valid: bool)
 -> (Option<Action>, Option<Page>) {
    widget::card(ui, |ui| {
        Flex::horizontal()
            .w_full()
            .gap([4.0, 0.0].into())
            .align_items(FlexAlign::Center)
            .show(ui, |flex| {
                task_card_content(flex, task, status, is_running, is_valid)
            })
            .inner
    })
}

fn task_card_content(
    flex: &mut FlexInstance,
    task: &Task,
    status: TaskStatus,
    is_running: bool,
    is_valid: bool)
 -> (Option<Action>, Option<Page>) {
    let mut action = None;
    let mut next_page = None;

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
    flex
        .add_ui(item(), |ui| {
           widget::action_button(
                ui,
                !is_running && is_valid,
                icon::PLAY,
                "Run")
        })
        .inner
        .clicked()
        .then(|| action = Some(Action::RunTask(task.id)));

    // Stop — disabled when not running.
    flex
        .add_ui(item(), |ui| {
            widget::action_button(
                ui,
                is_running,
                icon::STOP,
                "Stop")
        })
        .inner
        .clicked()
        .then(|| action = Some(Action::StopTask(task.id)));

    // Edit — disabled when running.
    flex
        .add_ui(item(), |ui| {
            widget::action_button(
                ui,
                !is_running,
                icon::PENCIL,
                "Edit")
        })
        .inner
        .clicked()
        .then(|| next_page = Some(Page::TaskEditor(task.clone())));

    flex
        .add(
            item().grow(1.0),
            Button::new("")
                .left_text(&task.name)
                .right_text(status_ui)
                .truncate())
        .clicked()
        .then(|| next_page = Some(Page::TaskViewer(task.id)));

    (action, next_page)
}
