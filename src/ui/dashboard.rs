use egui::*;

use crate::color;
use crate::icon;
use crate::data::TaskId;
use crate::engine::TaskEngine;
use crate::worker::TaskStatus;

use super::*;

/// What the dashboard wants the App layer to do after this frame.
///
/// `run_task` / `stop_task` only need `&mut TaskEngine` and are applied
/// inside `dashboard_ui` itself. The Edit and New actions are returned
/// instead because they need to mutate fields on `App` (`task_edit`,
/// `page`) that the dashboard does not own.
pub enum DashboardAction {
    None,
    /// Open the editor on a clone of an existing task.
    EditTask(TaskId),
    /// Open the editor on a fresh blank task.
    NewTask,
}

pub fn dashboard_ui(ui: &mut Ui, engine: &mut TaskEngine) -> PageResult {
    /// Fixed visual rhythm for every row. Kept as constants (rather than
    /// derived from text size) so disabled/enabled buttons don't reflow the
    /// row and so columns line up across the list.
    const ROW_H:    f32 = 24.0;
    const BTN_W:    f32 = 48.0;
    const STATUS_W: f32 = 64.0;

    struct TaskRow {
        id:         TaskId,
        name:       String,
        /// Pre-formatted status badge — owns its data so it can outlive the
        /// immutable borrow of `engine`.
        status:     RichText,
        is_running: bool,
        can_run:    bool,
    }

    // Phase 1: collect display snapshot while engine is immutably borrowed.
    // `TaskStatus<'_>` borrows from the worker and cannot be stored, so we
    // eagerly format it into an owned `RichText` here.
    let tasks: Vec<TaskRow> = engine.tasks()
        .map(|w| TaskRow {
            id:         w.task().id,
            name:       w.task().name.clone(),
            status:     format_task_status(&w.status(engine.plugins())),
            is_running: w.is_running(),
            can_run:    !w.is_running() && w.is_valid(engine.plugins()),
        })
        .collect();

    let mut action_edit: Option<TaskId> = None;
    let mut action_run:  Option<TaskId> = None;
    let mut action_stop: Option<TaskId> = None;

    // Phase 2: render using only the snapshot (no borrow on engine).
    ScrollArea::vertical().show(ui, |ui| {
        for row in &tasks {
            Frame::new()
                .fill(ui.visuals().faint_bg_color)
                .corner_radius(6.0)
                .inner_margin(Margin::symmetric(10, 6))
                .show(ui, |ui| {
                    // `right_to_left` grows from the right edge leftward, so
                    // each `add_*` call lands one slot further left. The name
                    // label sits inside a nested `left_to_right` so it hugs
                    // the leftmost remaining space and `truncate()` clamps to
                    // its inner width instead of the right-anchored cursor.
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui
                            .add_sized([BTN_W, ROW_H], Button::new(icon::PENCIL))
                            .on_hover_text("Edit")
                            .clicked()
                        {
                            action_edit = Some(row.id);
                        }

                        // Stop — disabled when not running.
                        if ui.add_enabled_ui(row.is_running, |ui| {
                            ui.add_sized([BTN_W, ROW_H], Button::new(icon::STOP))
                        }).inner.on_hover_text("Stop").clicked() {
                            action_stop = Some(row.id);
                        }

                        // Run — disabled while already running or invalid.
                        if ui.add_enabled_ui(row.can_run, |ui| {
                            ui.add_sized([BTN_W, ROW_H], Button::new(icon::PLAY))
                        }).inner.on_hover_text("Run").clicked() {
                            action_run = Some(row.id);
                        }

                        ui.add_sized(
                            [STATUS_W, ROW_H],
                            Label::new(row.status.clone()));

                        ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                            ui.add(Label::new(&row.name).truncate());
                        });
                    });
                });
        }
    });

    // Phase 3: apply engine-only actions (immutable borrow on engine is fully
    // released). Actions that touch App state are returned to the caller.
    if let Some(id) = action_run {
        engine.run_task(id)
            .unwrap_or_else(|err| log::error!("run_task {id}: {err:?}"));
    }
    if let Some(id) = action_stop { engine.stop_task(id); }

    if let Some(id) = action_edit {
        (None, Some(PageNavigation::TaskEditor(id)))
    } else {
        (None, None)
    }
}

fn format_task_status(status: &TaskStatus) -> RichText {
    match *status {
        TaskStatus::Invalid =>
            RichText::new("Invalid").color(color::ORANGE),
        TaskStatus::Stopped =>
            RichText::new("").weak(),
        TaskStatus::Running =>
            RichText::new("Running").color(color::BLUE),
        TaskStatus::Success(_) =>
            RichText::new("Success").color(color::GREEN),
        TaskStatus::Failure(_) =>
            RichText::new("Failed").color(color::RED),
    }.small()
}
