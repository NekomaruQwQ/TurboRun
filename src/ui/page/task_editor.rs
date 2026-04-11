use std::collections::BTreeMap;

use egui::*;

use crate::data::*;
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
    plugins: &BTreeMap<String, PluginPack>,
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
            // `base.nu / noop` is the deliberate no-op placeholder that newly
            // added rows start from — picked over "first available plugin"
            // because it's the one that does nothing if the user doesn't
            // bother to swap it.
            task.plugins.push(PluginInstance::new("base.nu", "noop"));
        }
    });

    // Deferred row-level mutations collected during the render loop, applied
    // after the borrow on `task.plugins` is released.
    let mut to_remove_plugin: Option<usize> = None;
    let mut to_move_up:       Option<usize> = None;
    let mut to_move_down:     Option<usize> = None;

    for (idx, inst) in task.plugins.iter_mut().enumerate() {
        // push_id keeps each row's child widget IDs stable as plugins are
        // added/removed/reordered at other indices.
        ui.push_id(idx, |ui| {
            ui.group(|ui| {
                // — Header row —
                ui.horizontal(|ui| {
                    ui.checkbox(&mut inst.enabled, "")
                        .on_hover_text("Enabled");

                    ui.label(nf::fa::FA_PUZZLE_PIECE);

                    let known_before =
                        plugins
                            .get(&inst.pack)
                            .and_then(|pack| pack.plugins.get(&inst.name))
                            .is_some();
                    let selected_text = if known_before {
                        RichText::new(format!("{} / {}", inst.pack, inst.name))
                    } else {
                        RichText::new(format!("{} / {} (missing)", inst.pack, inst.name))
                            .color(color::ORANGE)
                    };

                    ComboBox::from_id_salt("plugin_select")
                        .selected_text(selected_text)
                        .show_ui(ui, |ui| {
                            for (pack_name, pack) in plugins {
                                for (plugin_name, plugin) in &pack.plugins {
                                    let selected =
                                        inst.pack == *pack_name &&
                                        inst.name == *plugin_name;
                                    let label = format!("{pack_name} / {plugin_name}");
                                    if ui.selectable_label(selected, label).clicked() && !selected {
                                        inst.pack.clone_from(pack_name);
                                        inst.name.clone_from(plugin_name);
                                        // Drop any args/flags that the new plugin no longer
                                        // declares — silently keeping them would leak orphan
                                        // entries into the saved config. Args/flags whose names
                                        // exist on both sides are preserved on purpose so that
                                        // common conventions (e.g. `--unit`) survive a swap.
                                        inst.args.retain(|key, _| plugin.args.iter().any(|a| &a.name == key));
                                        inst.flags.retain(|f| plugin.flags.iter().any(|pf| &pf.name == f));
                                    }
                                }
                            }
                        });

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.small_button(nf::fa::FA_XMARK).on_hover_text("Remove plugin").clicked() {
                            to_remove_plugin = Some(idx);
                        }
                        if ui.small_button(nf::fa::FA_ARROW_DOWN).on_hover_text("Move down").clicked() {
                            to_move_down = Some(idx);
                        }
                        if ui.small_button(nf::fa::FA_ARROW_UP).on_hover_text("Move up").clicked() {
                            to_move_up = Some(idx);
                        }
                    });
                });

                // Re-look-up the plugin def *after* the selector so that
                // changing the selection takes effect this frame and we
                // don't render args/flags that the just-pruned `inst` no
                // longer carries.
                let Some(plugin) =
                    plugins
                        .get(&inst.pack)
                        .and_then(|pack| pack.plugins.get(&inst.name))
                else {
                    return;
                };

                // — Args grid —
                if !plugin.args.is_empty() {
                    Grid::new("plugin_args")
                        .num_columns(2)
                        .spacing([12.0, 4.0])
                        .show(ui, |ui| {
                            for arg in &plugin.args {
                                ui.horizontal(|ui| {
                                    let label_resp = ui.label(&arg.name);
                                    if !arg.optional {
                                        ui.label(RichText::new("*").color(color::RED));
                                    }
                                    if let Some(desc) = arg.description.as_deref() {
                                        let _ = label_resp.on_hover_text(desc);
                                    }
                                });

                                // Buffered edit: bind the widget to a local
                                // string and only write back on change. This
                                // avoids materializing empty entries into
                                // `inst.args` for every optional arg the user
                                // never touched, keeping the saved TOML clean.
                                let mut value: String =
                                    inst.args.get(&arg.name).cloned().unwrap_or_default();
                                let changed = match arg.accepted_values {
                                    None => {
                                        ui.add(
                                            TextEdit::singleline(&mut value)
                                                .desired_width(f32::INFINITY))
                                            .changed()
                                    }
                                    Some(ref accepted) => {
                                        let mut changed = false;
                                        ComboBox::from_id_salt(arg.name.as_str())
                                            .selected_text(&value)
                                            .show_ui(ui, |ui| {
                                                for choice in accepted {
                                                    if ui.selectable_value(&mut value, choice.clone(), choice).changed() {
                                                        changed = true;
                                                    }
                                                }
                                            });
                                        changed
                                    }
                                };
                                if changed {
                                    if value.is_empty() {
                                        inst.args.remove(&arg.name);
                                    } else {
                                        inst.args.insert(arg.name.clone(), value);
                                    }
                                }
                                ui.end_row();
                            }
                        });
                }

                // — Flags row —
                if !plugin.flags.is_empty() {
                    ui.horizontal_wrapped(|ui| {
                        for flag in &plugin.flags {
                            let was = inst.flags.contains(&flag.name);
                            let mut on = was;
                            let resp = ui.checkbox(&mut on, &flag.name);
                            if let Some(desc) = flag.description.as_deref() {
                                let _ = resp.on_hover_text(desc);
                            }
                            if on != was {
                                if on {
                                    inst.flags.push(flag.name.clone());
                                } else {
                                    inst.flags.retain(|f| f != &flag.name);
                                }
                            }
                        }
                    });
                }
            });
        });
    }

    // Apply deferred plugin mutations. Clicks are mutually exclusive within a
    // single render pass so the order of these branches doesn't matter.
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
