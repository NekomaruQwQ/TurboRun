use super::prelude::*;

pub fn task_viewer_ui(
    flex: &mut FlexInstance,
    view: &mut ViewContext,
    engine: &TaskEngine,
    task_id: TaskId) {
    let task = engine.task(task_id).expect("task_id must be valid");
    let stdout = engine.task_stdout(task_id);
    let stderr = engine.task_stderr(task_id);
    let status = engine.task_status(task_id);

    assert!(flex.is_vertical(), "task_viewer_ui requires a vertical flex");

    // — Main card —
    FlexCard::horizontal()
        .show(flex, |flex| task_main_card(flex, view, task, status));

    // — Command card (readonly) —
    FlexCard::vertical()
        .show(flex, |flex| {
            flex.add(item(), Label::new("Command"));
            flex.add(item(), Label::new(custom::code_block(&task.command)));
        });

    // — Plugins card (readonly) —
    FlexCard::vertical()
        .show(flex, |flex| task_plugin_card(flex, &task.plugins));

    // — Output cards —
    FlexCard::vertical()
        .show(flex, |flex| task_output_card(flex, "Standard Output", stdout));
    FlexCard::vertical()
        .show(flex, |flex| task_output_card(flex, "Standard Error", stderr));
}

fn task_main_card(
    flex: &mut FlexInstance,
    view: &mut ViewContext,
    task: &Task,
    status: TaskStatus) {
    flex.add_ui(
        item()
            .grow(1.0)
            .align_self_content(Align2::LEFT_CENTER),
        |ui| ui.horizontal(|ui| {
            ui.add_space(6.0);
            ui.add(Label::new(RichText::new(task.name.as_str()).heading()).wrap());
        }));
    flex.add(item(), Label::new(custom::task_status_label(status).small()));
    flex.add(item(), Label::new(""));
    flex.add_ui(item(), |ui| {
            ui.add_enabled(
                status != TaskStatus::Running &&
                status != TaskStatus::Invalid,
                Button::new(format!("{}  Start", nf::fa::FA_PLAY)))
        })
        .inner
        .on_hover_cursor(CursorIcon::PointingHand)
        .clicked()
        .then(|| view.set_action(Action::StartTask(task.id)));
    flex.add_ui(item(), |ui| {
            ui.add_enabled(
                status == TaskStatus::Running,
                Button::new(format!("{}  Stop", nf::fa::FA_STOP)))
        })
        .inner
        .on_hover_cursor(CursorIcon::PointingHand)
        .clicked()
        .then(|| view.set_action(Action::StopTask(task.id)));
    flex.add_ui(item(), |ui| {
            ui.add_enabled(
                status != TaskStatus::Running,
                Button::new(format!("{}  Edit", nf::fa::FA_PEN)))
        })
        .inner
        .on_hover_cursor(CursorIcon::PointingHand)
        .clicked()
        .then(|| view.set_navigation(Page::TaskEditor(task.clone())));
}

fn task_plugin_card(
    flex: &mut FlexInstance,
    plugins: &[PluginInstance]) {
    flex.add_ui(item(), |ui| ui.horizontal(|ui| {
        ui.label("Plugins");
        ui.label(RichText::new(format!("{} used", plugins.len())).weak().small());
    }));

    for plugin in plugins {
        flex.add_ui(item(), |ui| ui.vertical(|ui| {
            ui.label(
                format!(
                    "{}  {}::{}",
                    nf::fa::FA_PUZZLE_PIECE,
                    &plugin.pack,
                    &plugin.name));
            for (arg, value) in &plugin.args {
                ui.label(
                    RichText::new(format!("    --{arg} \"{value}\""))
                        .small()
                        .monospace());
            }
            for flag in &plugin.flags {
                ui.label(
                    RichText::new(format!("    --{flag}"))
                        .small()
                        .monospace());
            }
        }));
    }
}

fn task_output_card(
    flex: &mut FlexInstance,
    title: &str,
    lines: &[String]) {
    flex.add(item(), Label::new(title));

    // Here `Flex::add_ui` is needed to wrap the code block.
    flex.add_ui(item(), |ui| {
        if !lines.is_empty() {
            ui.label(custom::code_block(&lines.join("\n")));
        } else {
            ui.label(custom::code_block("(no output)").weak());
        }
    });
}
