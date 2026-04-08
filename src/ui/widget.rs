use tap::prelude::*;

use egui::*;
use egui_flex::*;
use derive_setters::Setters;

pub fn card<R, F>(ui: &mut Ui, body: F) -> R
where
    F: FnOnce(&mut Ui) -> R {
    use super::color;

    Frame::new()
        .fill(color::CARD)
        .corner_radius(6.0)
        .inner_margin(Margin::same(4))
        .show(ui, body)
        .inner
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[derive(Setters)]
pub struct ActionButton<'a> {
    pub icon: &'a str,
    pub tooltip: &'a str,
    pub enabled: bool,
    pub selected: bool,
}

impl ActionButton<'_> {
    pub const fn new() -> Self {
        Self {
            icon: "",
            tooltip: "",
            enabled: true,
            selected: false,
        }
    }
}

impl Widget for ActionButton<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        Button::new(self.icon)
            .selected(self.selected)
            .pipe(|body| |ui: &mut Ui| {
                ui.style_mut().spacing.button_padding = Vec2::ZERO;
                ui.add(body).on_hover_text(self.tooltip)
            })
            .pipe(|body| |ui: &mut Ui| ui.add_enabled(self.enabled, body))
            .pipe(|body| |ui: &mut Ui| ui.add_sized([22.0, 22.0], body))
            .pipe(|body| body(ui))
    }
}

#[expect(clippy::renamed_function_params, reason = "shorter names")]
impl FlexWidget for ActionButton<'_> {
    type Response = Response;

    fn flex_ui(self, item: FlexItem, flex: &mut FlexInstance) -> Self::Response {
        flex.add_ui(
            item,
            Button::new(self.icon)
                .selected(self.selected)
                .pipe(|body| |ui: &mut Ui| {
                    ui.style_mut().spacing.button_padding = Vec2::ZERO;
                    ui
                        .add(body)
                        .on_hover_cursor(CursorIcon::PointingHand)
                        .on_hover_text(self.tooltip)
                })
                .pipe(|body| |ui: &mut Ui| ui.add_enabled(self.enabled, body))
                .pipe(|body| |ui: &mut Ui| ui.add_sized([22.0, 22.0], body)))
            .inner
    }
}
