use egui::*;

use super::*;
use super::common::code_block;

pub fn plugin_ui(ui: &mut Ui, view: &mut ViewContext, engine: &TaskEngine) {
    ui
        .button(format!("{}  Reload Plugins", nf::fa::FA_ARROWS_ROTATE))
        .clicked()
        .then(|| view.set_action(Action::RefreshPlugins));

    ui.add_space(10.0);

    ScrollArea::vertical().show(ui, |ui| {
        // for plugin in engine.plugins_sorted() {
        //     let header = RichText::new(format!(
        //         "{}  {}",
        //         nf::fa::FA_PUZZLE_PIECE,
        //         plugin.name)).monospace();
        //     CollapsingHeader::new(header).show(ui, |ui| {
        //         ui.label(code_block(&plugin.source).weak());
        //     });
        // }
    });
}
