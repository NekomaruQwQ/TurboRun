use tap::prelude::*;

use egui::*;
use egui_flex::*;

use super::*;
use super::widget::*;

pub fn plugin_ui(
    flex: &mut FlexInstance,
    view: &mut ViewContext,
    engine: &TaskEngine) {
    FlexCard::horizontal()
        .padding(
            Margin::same(4)
                .tap_mut(|margin| margin.left += 4))
        .show(flex, |flex| {
            flex.add_ui(
                item()
                    .grow(1.0)
                    .align_self_content(Align2::LEFT_CENTER),
                |ui| ui.heading("Plugins"));
            flex.add(
                item(),
                Button::new(
                    format!("{}  Reload Plugins", nf::fa::FA_ARROWS_ROTATE)))
                .clicked()
                .then(|| view.set_action(Action::RefreshPlugins));
        });

    for plugin_pack in engine.plugin_packs().values() {
        FlexCard::vertical()
            .padding(Margin::symmetric(10, 6))
            .show(flex, |flex| {
                flex.add_ui(item(), |ui| ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(plugin_pack.name.as_str())
                            .monospace()
                            .heading());
                    ui.label(
                        RichText::new(plugin_pack.path.display().to_string())
                            .monospace()
                            .weak());
                }));

                for plugin in &plugin_pack.plugins {
                    flex.add_ui(item(), |ui| {
                        ui.monospace(format!("{} {}", nf::fa::FA_PUZZLE_PIECE, plugin.name));
                        ui.label(RichText::new(&plugin.description).small().weak());
                    });
                }
            });
    }
}
