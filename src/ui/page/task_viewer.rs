use egui::*;
use egui_flex::*;

use crate::data::*;
use crate::engine::*;

use super::*;
use super::widget::*;
use super::common::*;

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
    FlexCard::default()
        .stretch()
        .show(flex, |flex| task_main_card(flex, view, task, status));

    // — Command card (readonly) —
    FlexCard::default()
        .padding(Margin::symmetric(10, 8))
        .show(flex, |flex| {
            flex.add(item(), Label::new("Command"));
            flex.add(item(), Label::new(code_block(&task.command)));
        });

    // — Plugins card (readonly) —
    FlexCard::default()
        .padding(Margin::symmetric(10, 8))
        .show(flex, |flex| task_plugin_card(flex, &task.plugins));

    // — Output cards —
    FlexCard::default()
        .item(item().grow(1.0))
        .padding(Margin::symmetric(10, 8))
        .show(flex, |flex| task_output_card(flex, "Standard Output", stdout));
    FlexCard::default()
        .item(item().grow(2.0))
        .padding(Margin::symmetric(10, 8))
        .show(flex, |flex| task_output_card(flex, "Standard Error", stderr));
}

fn task_main_card(
    flex: &mut FlexInstance,
    view: &mut ViewContext,
    task: &Task,
    status: TaskStatus) {
    flex.add_flex(
        item(),
        Flex::horizontal()
            .w_full()
            .gap((4.0, 4.0).into()),
        |flex| {
            flex.add_ui(
                item()
                    .grow(1.0)
                    .align_self_content(Align2::LEFT_CENTER),
                |ui| ui.horizontal(|ui| {
                    ui.add_space(6.0);
                    ui.add(Label::new(RichText::new(task.name.as_str()).heading()).wrap());
                }));
            flex.add(item(), Label::new(task_status_label(status).small()));
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
        });
}

fn task_plugin_card(
    flex: &mut FlexInstance,
    plugins: &[PluginInstance]) {
    flex.add_ui(item(), |ui| ui.horizontal(|ui| {
        ui.label("Plugins");
        ui.label(RichText::new(format!("{} used", plugins.len())).weak().small());
    }));

    for inst in plugins {
        flex.add_ui(item(), |ui| {
            // let label = format!("{} {}", nf::fa::FA_PUZZLE_PIECE, &inst.item_name);
            // if super::TASK_VIEWER_PLUGIN_CARD_COMPACT {
            //     let vars =
            //         inst.vars
            //             .iter()
            //             .map(|&(ref key, ref value)| format!("{key}: \"{value}\""))
            //             .collect::<Vec<_>>()
            //             .join(", ");
            //     ui.horizontal(|ui| {
            //         ui.label(RichText::new(label).monospace());
            //         ui.label(RichText::new(format!("{{ {vars} }}")).monospace().weak());
            //     });
            // } else {
            //     CollapsingHeader::new(RichText::new(label).monospace()).show(ui, |ui| {
            //         for &(ref key, ref value) in &inst.vars {
            //             ui.horizontal(|ui| {
            //                 ui.label(RichText::new(key).monospace().weak());
            //                 ui.label(RichText::new(value).monospace());
            //             });
            //         }
            //     });
            // }
        });
    }
}

fn task_output_card(
    flex: &mut FlexInstance,
    title: &str,
    lines: &[String]) {
    flex.add(item(), Label::new(title));
    if !lines.is_empty() {
        flex.add(item(), Label::new(code_block(&lines.join("\n")).weak()));
    } else {
        flex.add(item(), Label::new(code_block("(no output)").weak()));
    }
}
