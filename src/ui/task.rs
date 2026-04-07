use egui::*;

use crate::color;
use crate::data::TaskId;
use crate::engine::TaskEngine;
use crate::icon;
use crate::worker::TaskStatus;

pub fn task_ui(ui: &mut Ui, engine: &mut TaskEngine, task_id: TaskId) -> super::PageResult {
    // Phase 1: render UI using immutable access, collect interaction results.
    let (did_start, did_stop) = {
        let plugins = engine.plugins();
        let worker = engine.task(task_id).expect("task_id must be valid");
        let task = worker.task();
        let status = worker.status(plugins);
        let is_running = worker.is_running();
        let is_valid = worker.is_valid(plugins);

        // — Heading + controls card —
        let (did_start, did_stop) = card(ui, |ui| {
            ui.heading(&task.name);
            ui.horizontal(|ui| {
                let s = ui.add_enabled(
                    !is_running && is_valid,
                    Button::new(format!("{}  Start", icon::PLAY))).clicked();
                let t = ui.add_enabled(
                    is_running,
                    Button::new(format!("{}  Stop", icon::STOP))).clicked();
                ui.label(format_status(&status));
                (s, t)
            }).inner
        });

        ui.add_space(8.0);

        // — Fields card (readonly) —
        card(ui, |ui| {
            Grid::new("task_fields")
                .num_columns(2)
                .spacing([12.0, 4.0])
                .show(ui, |ui| {
                    ui.label("Command");
                    ui.label(&task.command);
                    ui.end_row();

                    ui.label("Plugins");
                    if task.plugins.is_empty() {
                        ui.weak("(none)");
                    } else {
                        ui.vertical(|ui| {
                            for inst in &task.plugins {
                                if inst.vars.is_empty() {
                                    ui.label(&inst.name);
                                } else {
                                    let vars = inst.vars
                                        .iter()
                                        .map(|(k, v)| format!("{k}={v}"))
                                        .collect::<Vec<_>>()
                                        .join(", ");
                                    ui.label(format!("{} ({})", inst.name, vars));
                                }
                            }
                        });
                    }
                    ui.end_row();
                });
        });

        ui.add_space(8.0);

        // — stdout card —
        let stdout = worker.stdout();
        card(ui, |ui| {
            CollapsingHeader::new(format!("stdout ({} lines)", stdout.len()))
                .default_open(true)
                .show(ui, |ui| {
                    if stdout.is_empty() {
                        ui.weak("(no output)");
                    } else {
                        ScrollArea::vertical()
                            .id_salt("stdout_scroll")
                            .max_height(160.0)
                            .stick_to_bottom(true)
                            .show(ui, |ui| {
                                for line in stdout {
                                    ui.monospace(line);
                                }
                            });
                    }
                });
        });

        ui.add_space(8.0);

        // — stderr card —
        let stderr = worker.stderr();
        // Auto-expand stderr when it has content, since non-empty stderr usually
        // signals an error worth surfacing immediately.
        card(ui, |ui| {
            CollapsingHeader::new(format!("stderr ({} lines)", stderr.len()))
                .default_open(!stderr.is_empty())
                .show(ui, |ui| {
                    if stderr.is_empty() {
                        ui.weak("(no output)");
                    } else {
                        ScrollArea::vertical()
                            .id_salt("stderr_scroll")
                            .max_height(160.0)
                            .stick_to_bottom(true)
                            .show(ui, |ui| {
                                for line in stderr {
                                    ui.monospace(line);
                                }
                            });
                    }
                });
        });

        (did_start, did_stop)
    };

    // Phase 2: mutations (immutable borrow is now released).
    if did_start {
        engine.run_task(task_id)
            .unwrap_or_else(|err| log::error!("run_task failed: {err:?}"));
    }
    if did_stop {
        engine.stop_task(task_id);
    }

    (None, None)
}

/// Wraps a UI section in a PWA-style "card": rounded, padded, painted with
/// the theme's faint surface color. Local helper because `task.rs` is the only
/// consumer; promote to `theme::card` if a second site appears.
fn card<R>(ui: &mut Ui, body: impl FnOnce(&mut Ui) -> R) -> R {
    Frame::new()
        .fill(ui.visuals().faint_bg_color)
        .corner_radius(6.0)
        .inner_margin(Margin::same(10))
        .show(ui, body)
        .inner
}

fn format_status(status: &TaskStatus<'_>) -> RichText {
    match status {
        TaskStatus::Invalid   => RichText::new("Invalid").color(color::ORANGE),
        TaskStatus::Stopped   => RichText::new("").weak(),
        TaskStatus::Running   => RichText::new("Running").color(color::BLUE),
        TaskStatus::Success(_) => RichText::new("Success").color(color::GREEN),
        TaskStatus::Failure(_) => RichText::new("Failed").color(color::RED),
    }.small()
}
