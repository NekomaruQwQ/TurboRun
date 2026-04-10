use std::time::SystemTime;

use egui::*;

use super::color;
use crate::data::*;
use crate::data::Plugin;

use super::*;

/// Renders the task editor page for `editor` and reports the user's intent
/// for this frame via `page`. The caller is responsible for performing any
/// engine-side mutations (insert / replace / remove + `save_config`) when
/// the action is later applied.
///
/// `is_existing` controls the heading and whether the Delete button is shown.
/// It is computed by the caller because we hold no mutable borrow of `engine`
/// here and the App layer already knows the answer.
#[expect(clippy::too_many_lines, reason = "UI code is inherently verbose")]
pub fn task_editor_ui(
    ui: &mut Ui,
    view: &mut ViewContext,
    plugins: &PluginMap,
    task: &mut Task,
    is_existing: bool) {
    ui.separator();

    // — Action row —
    // Validation mirrors `TaskWorker::is_valid`: we only check the inputs
    // the user is editing here. Plugin existence is also surfaced inline via
    // the "(missing)" label, but we don't gate Save on it — letting the user
    // save a task pointing at a not-yet-loaded plugin is consistent with how
    // the rest of the app handles missing plugins (mark Invalid, allow edit).
    let valid =
        !task.name.trim().is_empty() &&
        !task.command.trim().is_empty();

    // Copied out so the inner `with_layout` closure can reference the id
    // without re-borrowing `task` (which the outer closure already holds
    // mutably for `last_modified` / `clone`).
    let task_id = task.id;

    ui.horizontal(|ui| {
        if ui.add_enabled(valid, Button::new(format!("{}  Save", nf::fa::FA_FLOPPY_DISK))).clicked() {
            view.set_action(Action::SaveTask(task.clone()));
            view.set_navigation(Page::TaskViewer(task_id));
        }

        if ui.button(format!("{}  Cancel", nf::fa::FA_XMARK)).clicked() {
            view.set_navigation(if is_existing {
                Page::TaskViewer(task_id)
            } else {
                Page::Dashboard
            });
        }

        if is_existing {
            // Right-aligned two-click delete confirm. State is stashed in
            // egui memory keyed by task id so it survives across frames but
            // is automatically cleared the next time `Memory` is wiped (i.e.
            // never within a session — we explicitly clear on commit).
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let confirm_id = Id::new(("edit_task_delete_confirm", task_id));
                let armed: bool = ui
                    .data_mut(|d| d.get_temp::<bool>(confirm_id))
                    .unwrap_or(false);

                let label = if armed {
                    RichText::new(format!("{}  Delete?", nf::fa::FA_TRASH)).color(color::RED)
                } else {
                    RichText::new(format!("{}  Delete", nf::fa::FA_TRASH))
                };
                if ui.button(label).clicked() {
                    if armed {
                        ui.data_mut(|d| d.remove::<bool>(confirm_id));
                        view.set_action(Action::DeleteTask(task_id));
                        view.set_navigation(Page::Dashboard);
                    } else {
                        ui.data_mut(|d| d.insert_temp(confirm_id, true));
                    }
                }
            });
        }
    });

    ui.separator();

    // — Name + Command —
    Grid::new("edit_task_fields")
        .num_columns(2)
        .spacing([12.0, 4.0])
        .show(ui, |ui| {
            ui.label("Name");
            ui.add(
                TextEdit::singleline(&mut task.name)
                    .desired_width(f32::INFINITY));
            ui.end_row();

            ui.label("Command");
            ui.add(
                TextEdit::multiline(&mut task.command)
                    .code_editor()
                    .desired_rows(4)
                    .desired_width(f32::INFINITY));
            ui.end_row();
        });

    ui.separator();

    // — Plugins section —
    ui.horizontal(|ui| {
        ui.label("Plugins");
        if ui.small_button(format!("{}  Add plugin", nf::fa::FA_PLUS)).clicked() {
            // Default to the first available plugin name; if none are loaded
            // we still let the user add a row, which will surface as missing
            // and prompt them to fix the plugin directory / config.
            task.plugins.push(PluginInstance::new("base.nu".into(), "noop".into()));
        }
    });

    // Deferred mutations collected during the render loop, applied below.
    let mut to_remove_plugin: Option<usize> = None;
    let mut to_move_up:       Option<usize> = None;
    let mut to_move_down:     Option<usize> = None;

    for (idx, inst) in task.plugins.iter_mut().enumerate() {
        // push_id keeps each plugin row's child widget IDs stable as plugins
        // are added/removed at other indices.
        // ui.push_id(idx, |ui| {
        //     ui.group(|ui| {
        //         ui.horizontal(|ui| {
        //             let missing =
        //                 !plugins
        //                     .iter()
        //                     .any(|&plugin| {
        //                         plugin.file_name == inst.file_name &&
        //                         plugin.item_name == inst.item_name
        //                     });
        //             let label = if missing {
        //                 RichText::new(format!("{} (missing)", inst.item_name))
        //                     .color(color::ORANGE)
        //             } else {
        //                 RichText::new(&inst.item_name)
        //             };

        //             ComboBox::from_id_salt("plugin_combo")
        //                 .selected_text(label)
        //                 .show_ui(ui, |ui| {
        //                     for plugin in plugins {
        //                         ui.selectable_value(&mut inst.item_name, plugin.name.clone(), &plugin.name);
        //                     }
        //                 });

        //             if ui.small_button(nf::fa::FA_ARROW_UP).on_hover_text("Move up").clicked() {
        //                 to_move_up = Some(idx);
        //             }
        //             if ui.small_button(nf::fa::FA_ARROW_DOWN).on_hover_text("Move down").clicked() {
        //                 to_move_down = Some(idx);
        //             }
        //             if ui.small_button(nf::fa::FA_XMARK).on_hover_text("Remove plugin").clicked() {
        //                 to_remove_plugin = Some(idx);
        //             }
        //         });

        //         ui.weak("keys must match {{name}} placeholders in the plugin source");

        //         for (row_idx, &mut (ref mut key, ref mut value)) in inst.vars.iter_mut().enumerate() {
        //             ui.push_id(row_idx, |ui| {
        //                 ui.horizontal(|ui| {
        //                     ui.add(
        //                         TextEdit::singleline(key)
        //                             .hint_text("name")
        //                             .desired_width(120.0));
        //                     ui.add(
        //                         TextEdit::singleline(value)
        //                             .hint_text("value")
        //                             .desired_width(180.0));
        //                     if ui.small_button(nf::fa::FA_XMARK).on_hover_text("Remove var").clicked() {
        //                         to_remove_var = Some((idx, row_idx));
        //                     }
        //                 });
        //             });
        //         }

        //         if ui.small_button(format!("{}  Add var", nf::fa::FA_PLUS)).clicked() {
        //             to_add_var = Some(idx);
        //         }
        //     });
        // });
    }

    // Apply deferred plugin/var mutations. Order matters only insofar as we
    // never apply two conflicting actions in the same frame — clicks are
    // mutually exclusive within a single render pass.
    if let Some(i) = to_remove_plugin {
        task.plugins.remove(i);
    }
    if let Some(i) = to_move_up
        && i > 0
    {
        task.plugins.swap(i, i - 1);
    }
    if let Some(i) = to_move_down
        && i + 1 < task.plugins.len()
    {
        task.plugins.swap(i, i + 1);
    }
}
