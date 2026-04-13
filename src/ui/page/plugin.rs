use tap::prelude::*;

use egui::*;
use egui_flex::*;

use super::*;
use super::widget::*;
use super::common::code_block;

pub fn plugin_ui(ui: &mut Ui, view: &mut ViewContext, engine: &TaskEngine) {
    Flex::vertical()
        .w_full()
        .gap((8.0, 8.0).into())
        .show(ui, |flex| {
            FlexCard::default()
                .stretch()
                .padding(
                    Margin::same(4)
                        .tap_mut(|margin| margin.left += 4))
                .show(flex, |flex| {
                    flex.add_flex(
                        item(),
                        Flex::horizontal()
                            .w_full()
                            .gap((4.0, 4.0).into()),
                        |flex| {
                            flex.add(
                                item(),
                                Label::new(RichText::new("Plugins").heading()));
                            FlexSpace::fill(flex);
                            flex.add(
                                item(),
                                Button::new(
                                    format!("{}  Reload Plugins", nf::fa::FA_ARROWS_ROTATE)))
                                .clicked()
                                .then(|| view.set_action(Action::RefreshPlugins));
                        });
                });

            for plugin_pack in engine.plugin_packs().values() {
                FlexCard::default()
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
                            flex.add(
                                item(),
                                Label::new(
                                    RichText::new(format!("{} {}", nf::fa::FA_PUZZLE_PIECE, plugin.name))
                                        .monospace()));
                        }
                    });
            }
        });
}
