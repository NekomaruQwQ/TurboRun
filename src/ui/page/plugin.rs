use egui::*;

use super::*;

pub fn plugin_ui(ui: &mut Ui, view: &mut ViewContext, engine: &TaskEngine) {
    ui
        .button(format!("{}  Reload Plugins", nf::fa::FA_ARROWS_ROTATE))
        .clicked()
        .then(|| view.set_action(Action::RefreshPlugins));

    ui.add_space(10.0);

    ScrollArea::vertical().show(ui, |ui| {
        for plugin in engine.plugins_sorted() {
            let header = RichText::new(format!(
                "{}  {}",
                nf::fa::FA_PUZZLE_PIECE,
                plugin.name)).monospace();
            let source =
                RichText::new(plugin.source.trim_end())
                .monospace()
                .weak()
                .line_height(Some(15.0));
            CollapsingHeader::new(header).show(ui, |ui| {
                ui.label(source);
            });
        }
    });
}