use std::time::SystemTime;

use egui::*;

use crate::color;
use crate::data::*;
use crate::engine::TaskEngine;
use crate::icon;

/// Result of one frame of the task editor — consumed by the App layer to
/// decide what (if anything) to do with the underlying engine state. Returning
/// an action keeps `edit_task_ui` pure with respect to the engine and avoids
/// having to thread two simultaneous `&mut` borrows through the call.
pub enum EditAction {
    None,
    Save,
    Cancel,
    Delete,
}

/// Renders the task editor page for `editor` and returns the user's intent
/// for this frame. The caller is responsible for performing any engine-side
/// mutations (insert / replace / remove + `save_config`) afterwards.
///
/// `is_existing` controls the heading and whether the Delete button is shown.
/// It is computed by the caller because we hold no mutable borrow of `engine`
/// here and the App layer already knows the answer.
pub fn edit_task_ui(
    ui: &mut Ui,
    engine: &TaskEngine,
    editor: &mut TaskEditor,
    is_existing: bool) -> EditAction {
    let mut action = EditAction::None;

    ui.heading(if is_existing { "Edit Task" } else { "New Task" });
    ui.separator();

    // — Name + Command —
    Grid::new("edit_task_fields")
        .num_columns(2)
        .spacing([12.0, 4.0])
        .show(ui, |ui| {
            ui.label("Name");
            ui.add(
                TextEdit::singleline(&mut editor.task.name)
                    .desired_width(f32::INFINITY));
            ui.end_row();

            ui.label("Command");
            ui.add(
                TextEdit::multiline(&mut editor.task.command)
                    .code_editor()
                    .desired_rows(4)
                    .desired_width(f32::INFINITY));
            ui.end_row();
        });

    ui.separator();

    // — Plugins section —
    // Snapshot the available plugin names once so the inner combo boxes don't
    // need to re-borrow `engine` per row.
    let plugin_names: Vec<String> =
        engine.plugins_sorted()
            .map(|p| p.name.clone())
            .collect();

    ui.horizontal(|ui| {
        ui.label("Plugins");
        if ui.small_button(format!("{}  Add plugin", icon::PLUS)).clicked() {
            // Default to the first available plugin name; if none are loaded
            // we still let the user add a row, which will surface as missing
            // and prompt them to fix the plugin directory / config.
            let default_name = plugin_names
                .first()
                .cloned()
                .unwrap_or_default();
            editor.task.plugins.push(PluginInstance::new(&default_name));
            editor.var_rows.push(Vec::new());
        }
    });

    // Deferred mutations collected during the render loop, applied below.
    let mut to_remove_plugin: Option<usize> = None;
    let mut to_move_up:       Option<usize> = None;
    let mut to_move_down:     Option<usize> = None;
    let mut to_add_var:       Option<usize> = None;
    let mut to_remove_var:    Option<(usize, usize)> = None;

    // Split borrow so we can iterate plugins and var_rows in lockstep while
    // both are mutably accessible inside the loop.
    let plugins  = &mut editor.task.plugins;
    let var_rows = &mut editor.var_rows;

    for (idx, (inst, rows)) in plugins.iter_mut().zip(var_rows.iter_mut()).enumerate() {
        // push_id keeps each plugin row's child widget IDs stable as plugins
        // are added/removed at other indices.
        ui.push_id(idx, |ui| {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    let missing = !plugin_names.contains(&inst.name);
                    let label = if missing {
                        RichText::new(format!("{} (missing)", inst.name))
                            .color(color::ORANGE)
                    } else {
                        RichText::new(&inst.name)
                    };
                    ComboBox::from_id_salt("plugin_combo")
                        .selected_text(label)
                        .show_ui(ui, |ui| {
                            for name in &plugin_names {
                                ui.selectable_value(&mut inst.name, name.clone(), name);
                            }
                        });

                    if ui.small_button(icon::ARROW_UP).on_hover_text("Move up").clicked() {
                        to_move_up = Some(idx);
                    }
                    if ui.small_button(icon::ARROW_DOWN).on_hover_text("Move down").clicked() {
                        to_move_down = Some(idx);
                    }
                    if ui.small_button(icon::TIMES).on_hover_text("Remove plugin").clicked() {
                        to_remove_plugin = Some(idx);
                    }
                });

                ui.weak("keys must match {{name}} placeholders in the plugin source");

                for (row_idx, (key, value)) in rows.iter_mut().enumerate() {
                    ui.push_id(row_idx, |ui| {
                        ui.horizontal(|ui| {
                            ui.add(
                                TextEdit::singleline(key)
                                    .hint_text("name")
                                    .desired_width(120.0));
                            ui.add(
                                TextEdit::singleline(value)
                                    .hint_text("value")
                                    .desired_width(180.0));
                            if ui.small_button(icon::TIMES).on_hover_text("Remove var").clicked() {
                                to_remove_var = Some((idx, row_idx));
                            }
                        });
                    });
                }

                if ui.small_button(format!("{}  Add var", icon::PLUS)).clicked() {
                    to_add_var = Some(idx);
                }
            });
        });
    }

    // Apply deferred plugin/var mutations. Order matters only insofar as we
    // never apply two conflicting actions in the same frame — clicks are
    // mutually exclusive within a single render pass.
    if let Some(i) = to_remove_plugin {
        editor.task.plugins.remove(i);
        editor.var_rows.remove(i);
    }
    if let Some(i) = to_move_up
        && i > 0
    {
        editor.task.plugins.swap(i, i - 1);
        editor.var_rows.swap(i, i - 1);
    }
    if let Some(i) = to_move_down
        && i + 1 < editor.task.plugins.len()
    {
        editor.task.plugins.swap(i, i + 1);
        editor.var_rows.swap(i, i + 1);
    }
    if let Some(i) = to_add_var {
        editor.var_rows[i].push((String::new(), String::new()));
    }
    if let Some((i, j)) = to_remove_var {
        editor.var_rows[i].remove(j);
    }

    ui.separator();

    // — Action row —
    // Validation mirrors `TaskWorker::is_valid`: we only check the inputs
    // the user is editing here. Plugin existence is also surfaced inline via
    // the "(missing)" label, but we don't gate Save on it — letting the user
    // save a task pointing at a not-yet-loaded plugin is consistent with how
    // the rest of the app handles missing plugins (mark Invalid, allow edit).
    let valid =
        !editor.task.name.trim().is_empty() &&
        !editor.task.command.trim().is_empty();

    ui.horizontal(|ui| {
        if ui.add_enabled(valid, Button::new(format!("{}  Save", icon::SAVE))).clicked() {
            action = EditAction::Save;
        }
        if ui.button(format!("{}  Cancel", icon::TIMES)).clicked() {
            action = EditAction::Cancel;
        }

        if is_existing {
            // Right-aligned two-click delete confirm. State is stashed in
            // egui memory keyed by task id so it survives across frames but
            // is automatically cleared the next time `Memory` is wiped (i.e.
            // never within a session — we explicitly clear on commit).
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let confirm_id = Id::new(("edit_task_delete_confirm", editor.task.id));
                let armed: bool = ui
                    .data_mut(|d| d.get_temp::<bool>(confirm_id))
                    .unwrap_or(false);

                let label = if armed {
                    RichText::new(format!("{}  Delete?", icon::TRASH)).color(color::RED)
                } else {
                    RichText::new(format!("{}  Delete", icon::TRASH))
                };
                if ui.button(label).clicked() {
                    if armed {
                        action = EditAction::Delete;
                        ui.data_mut(|d| d.remove::<bool>(confirm_id));
                    } else {
                        ui.data_mut(|d| d.insert_temp(confirm_id, true));
                    }
                }
            });
        }
    });

    // Bump last_modified eagerly here so it's accurate when the App layer
    // baked the task into the engine on Save.
    if matches!(action, EditAction::Save) {
        editor.task.last_modified = SystemTime::now();
        editor.finalize();
    }

    action
}
