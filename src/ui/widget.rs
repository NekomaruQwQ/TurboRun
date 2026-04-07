use egui::*;

use crate::theme::*;

pub fn card<R, F>(ui: &mut Ui, body: F) -> R
where
    F: FnOnce(&mut Ui) -> R {
    Frame::new()
        .fill(COLOR_CARD)
        .corner_radius(6.0)
        .inner_margin(Margin::same(4))
        .show(ui, body)
        .inner
}

pub fn action_button(ui: &mut Ui, enabled: bool, icon: &str, tooltip: &str) -> Response {
    ui.add_sized([22.0, 22.0], |ui: &mut Ui| {
        ui.style_mut().spacing.button_padding = Vec2::ZERO;
        ui.add_enabled(enabled, Button::new(icon)).on_hover_text(tooltip)
    })
}
