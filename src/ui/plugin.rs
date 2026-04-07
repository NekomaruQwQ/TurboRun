use egui::*;

use crate::engine::TaskEngine;
use crate::icon;

pub fn plugins_ui(ui: &mut Ui, engine: &mut TaskEngine) -> super::PageResult {
    ui
        .button(format!("{}  Reload Plugins", icon::REFRESH))
        .clicked()
        .then(|| {
            engine
                .scan_plugins()
                .unwrap_or_else(|err| {
                    log::error!(
                        "failed to scan plugins in {}: {err:?}",
                        engine.plugin_dir().display());
                });
        });

    ui.add_space(10.0);

    ScrollArea::vertical().show(ui, |ui| {
        for plugin in engine.plugins_sorted() {
            let header = RichText::new(format!(
                "{}  {}",
                icon::PUZZLE_PIECE,
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

    (None, None)
}
