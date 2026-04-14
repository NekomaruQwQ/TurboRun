use super::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum PluginAction {
    Remove(usize),
    MoveUp(usize),
    MoveDown(usize),
}

pub fn task_editor_ui(
    flex: &mut FlexInstance,
    view: &mut ViewContext,
    engine: &TaskEngine,
    task: &mut Task) {
    let task_id = task.id;
    let task_exist = engine.task(task_id).is_some();
    let task_saved = engine.task(task_id) == Some(task);

    FlexCard::horizontal()
        .padding(
            Margin::same(4)
                .tap_mut(|margin| margin.left += 4))
        .show(flex, |flex| {
            flex.add_ui(
                item()
                    .grow(1.0)
                    .align_self_content(Align2::LEFT_CENTER),
                |ui| ui.heading(task.name.as_str()));
            flex.add_ui(item(), |ui| {
                    ui.add_enabled(
                        !task_saved,
                        Button::new(
                            format!("{}  Save", if !task_saved {
                                nf::fa::FA_FLOPPY_DISK
                            } else {
                                nf::fa::FA_CHECK
                            })))
                })
                .inner
                .on_hover_cursor(CursorIcon::PointingHand)
                .clicked()
                .then(|| view.set_action(Action::SaveTask(task.clone())));
            flex.add_ui(item(), |ui| {
                    ui.button(format!("{}  Cancel", nf::fa::FA_XMARK))
                })
                .inner
                .on_hover_cursor(CursorIcon::PointingHand)
                .clicked()
                .then(|| view.set_navigation(if task_exist {
                    Page::TaskViewer(task_id)
                } else {
                    Page::Dashboard
                }));

            if task_exist {
                // Right-aligned two-click delete confirm. State is stashed in
                // egui memory keyed by task id so it survives across frames but
                // is automatically cleared the next time `Memory` is wiped (i.e.
                // never within a session — we explicitly clear on commit).
                task_delete_button_ui(flex, view, task_id);
            }
        });

    FlexCard::vertical()
        .padding(
            Margin::same(4)
                .tap_mut(|margin| margin.top += 4))
        .show(flex, |flex| {
            flex.add(item(), Label::new("  Task Name"));
            flex.add_ui(
                item()
                    .grow(1.0)
                    .align_self_content(Align2::LEFT_CENTER),
                |ui| ui.add(
                    TextEdit::singleline(&mut task.name)
                        .desired_width(f32::INFINITY)));
            flex.add(item(), Label::new("  Task Command"));
            flex.add_ui(
                item(),
                |ui| ui.add(
                    TextEdit::multiline(&mut task.command)
                        .code_editor()
                        .desired_rows(4)
                        .desired_width(f32::INFINITY)));
        });

    let mut plugin_action = None;
    for (idx, inst) in task.plugins.iter_mut().enumerate() {
        flex.add_ui(item(), |ui| {
            ui.push_id(ui.auto_id_with(idx.to_string()), |ui| {
                FlexCard::vertical()
                    .padding(Margin::same(4))
                    .gap((4.0, 4.0).into())
                    .stretch()
                    .show_ui(ui, |flex| {
                        task_plugin_editor_ui(flex, engine, idx, inst)
                            .tap_some(|&action| plugin_action = Some(action));
                    });
            });
        });
    }

    match plugin_action {
        Some(PluginAction::Remove(i)) =>
            task.plugins.remove(i).pipe(|_| ()),
        Some(PluginAction::MoveUp(i)) if i > 0 =>
            task.plugins.swap(i, i - 1),
        Some(PluginAction::MoveDown(i)) if i + 1 < task.plugins.len() =>
            task.plugins.swap(i, i + 1),
        _ => (),
    }

    FlexCard::horizontal()
        .show(flex, |flex| {
            flex.add(
                    item().grow(1.0),
                    Button::new(format!("{}  New Plugin", nf::fa::FA_PLUS)))
                .on_hover_cursor(CursorIcon::PointingHand)
                .clicked()
                .then(|| task.plugins.push(PluginInstance::noop()));
        });
}

// TODO: This function was authored by Claude Code and may need some
// polishing on its behavior. The current implementation is a quick
// port of the old `egui::Ui::data_mut` based logic.
fn task_delete_button_ui(
    flex: &mut FlexInstance,
    view: &mut ViewContext,
    task_id: TaskId) {
    let confirm_id = Id::new(("edit_task_delete_confirm", task_id));
    let armed: bool =
        flex.ui()
            .data_mut(|d| d.get_temp::<bool>(confirm_id))
            .unwrap_or(false);

    let label = if armed {
        RichText::new(format!("{}  Delete", nf::fa::FA_QUESTION)).color(color::RED)
    } else {
        RichText::new(format!("{}  Delete", nf::fa::FA_TRASH))
    };

    if flex.add(item(), Button::new(label))
        .on_hover_cursor(CursorIcon::PointingHand)
        .clicked() {
        if armed {
            flex.ui().data_mut(|d| d.remove::<bool>(confirm_id));
            view.set_action(Action::DeleteTask(task_id));
            view.set_navigation(Page::Dashboard);
        } else {
            flex.ui().data_mut(|d| d.insert_temp(confirm_id, true));
        }
    }
}

fn task_plugin_editor_ui(
    flex: &mut FlexInstance,
    engine: &TaskEngine,
    inst_id: usize,
    inst: &mut PluginInstance)
 -> Option<PluginAction> {
    let action =
        flex.add_flex(
                item(),
                Flex::horizontal()
                    .w_full()
                    .align_items_content(Align2::LEFT_CENTER)
                    .gap((4.0, 4.0).into()),
                |flex| task_plugin_header_ui(flex, engine, inst_id, inst))
            .inner;

    if let Some(plugin) = engine.plugins().get(&inst.plugin()) && !(
        plugin.args.is_empty() &&
        plugin.flags.is_empty()) {
        flex.add_ui(item(), |ui| {
            Grid::new("plugin_args")
                .num_columns(3)
                .min_col_width(0.0)
                .spacing([8.0, 4.0])
                .show(ui, |ui| {
                    for flag in &plugin.flags {
                        task_plugin_label_ui(
                            ui,
                            &flag.name,
                            false,
                            &flag.description);

                        let mut selected = inst.flags.contains(&flag.name);
                        ui
                            .checkbox(&mut selected, "")
                            .on_hover_cursor(CursorIcon::PointingHand)
                            .clicked()
                            .then(|| {
                                if selected {
                                    inst.flags.remove(&flag.name);
                                } else {
                                    inst.flags.insert(flag.name.clone());
                                }
                            });
                        ui.end_row();
                    }

                    for arg in &plugin.args {
                        task_plugin_label_ui(
                            ui,
                            &arg.name,
                            !arg.optional,
                            &arg.description);
                        task_plugin_args_editor_ui(ui, arg, inst);
                        ui.end_row();
                    }
                });
        });
    }

    action
}

fn task_plugin_header_ui(
    flex: &mut FlexInstance,
    engine: &TaskEngine,
    inst_id: usize,
    inst: &mut PluginInstance)
 -> Option<PluginAction> {
    let mut action = None;

    flex.add_ui(item(), |ui| {
        ui.add_space(4.0);
        ui.heading(nf::fa::FA_PUZZLE_PIECE);
    });

    flex.add_ui(item().grow(1.0), |ui| {
        task_plugin_select_ui(ui, engine, inst);
    });

    flex.add(
            item(),
            Button::new(
                if inst.enabled {
                    RichText::new(format!("Enabled  {}", nf::fa::FA_CHECK))
                } else {
                    RichText::new(format!("Disabled  {}", nf::fa::FA_XMARK))
                        .color(color::RED)
                }))
        .clicked()
        .then(|| inst.enabled = !inst.enabled);
    flex.add(
            item(),
            FlexActionButton::new()
                .icon(nf::fa::FA_ARROW_UP))
        .on_hover_text("Move up")
        .clicked()
        .then(|| action = Some(PluginAction::MoveUp(inst_id)));
    flex.add(
            item(),
            FlexActionButton::new()
                .icon(nf::fa::FA_ARROW_DOWN))
        .on_hover_text("Move down")
        .clicked()
        .then(|| action = Some(PluginAction::MoveDown(inst_id)));
    flex.add(
            item(),
            FlexActionButton::new()
                .icon(nf::fa::FA_TRASH))
        .on_hover_text("Remove Plugin")
        .clicked()
        .then(|| action = Some(PluginAction::Remove(inst_id)));

    action
}

fn task_plugin_select_ui(
    ui: &mut egui::Ui,
    engine: &TaskEngine,
    inst: &mut PluginInstance) {
    let label =
        format!("{}::{}", inst.pack, inst.name);
    let exist =
        engine.plugins().contains_key(&inst.plugin());

    ComboBox::from_id_salt("plugin_select")
        .selected_text(
            if exist {
                RichText::new(label)
            } else {
                RichText::new(format!("{label} (missing)"))
                    .color(color::ORANGE)
            })
        .show_ui(ui, |ui| {
            for plugin_pack in engine.plugin_packs().values() {
                for plugin in &plugin_pack.plugins {
                    let selected =
                        inst.plugin() == (
                            plugin_pack.name.clone(),
                            plugin.name.clone());
                    ui.add(
                        Button::selectable(selected, "")
                            .left_text(
                                RichText::new(
                                    format!("{}::{}", plugin_pack.name, plugin.name))
                                    .monospace()))
                        .clicked()
                        .pipe(|clicked| clicked && !selected)
                        .then(|| *inst = PluginInstance {
                            pack: plugin_pack.name.clone(),
                            name: plugin.name.clone(),
                            enabled: inst.enabled,
                            ..PluginInstance::default()
                        });
                }
            }
        });
}

fn task_plugin_label_ui(
    ui: &mut Ui,
    name: &str,
    required: bool,
    description: &str) {
    ui.horizontal(|ui| {
        ui.style_mut().spacing.item_spacing.x = 2.0;
        ui.add_space(4.0);
        ui.label(RichText::new(format!("--{name}")).monospace());
        if required {
            ui.label(RichText::new("*").color(color::RED));
        }
    });

    if !description.is_empty() {
        ui
            .label(RichText::new(nf::oct::OCT_INFO).small().weak())
            .on_hover_text(description);
    } else {
        ui.label("");
    };
}

// TODO: This function was authored by Claude Code and may need a full
// rewrite to migrate to flex-based layout. The current implementation
//  is a quick port of the old `&mut egui::Ui` based layout.
fn task_plugin_args_editor_ui(
    ui: &mut egui::Ui,
    arg: &PluginArg,
    inst: &mut PluginInstance) {
    // Buffered edit: bind the widget to a local
    // `String` (TextEdit needs `&mut dyn TextBuffer`,
    // which `SmolStr` does not impl) and only write
    // back on change. This avoids materializing empty
    // entries into `inst.args` for every optional arg
    // the user never touched, keeping the saved TOML
    // clean.
    let mut value: String =
        inst.args.get(&arg.name).map(<_>::to_string).unwrap_or_default();
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
                .selected_text(
                    if !value.is_empty() {
                        RichText::new(value.clone())
                    } else {
                        RichText::new("(none)").monospace().weak()
                    })
                .show_ui(ui, |ui| {
                    if arg.optional {
                        ui
                            .selectable_value(
                                &mut value,
                                String::new(),
                                RichText::new("(none)").monospace().weak())
                            .changed()
                            .then(|| changed = true);
                    }
                    for choice in accepted {
                        if ui.selectable_value(&mut value, choice.to_string(), choice.as_str()).changed() {
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
            inst.args.insert(arg.name.clone(), <_>::from(value.as_str()));
        }
    }
}
