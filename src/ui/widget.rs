use tap::prelude::*;
use derive_setters::Setters;

use egui::*;
use egui_flex::*;

pub struct FlexSeparator;

impl FlexWidget for FlexSeparator {
    type Response = ();

    fn flex_ui(self, item: FlexItem, flex_instance: &mut FlexInstance) -> Self::Response {
        flex_instance.add_ui(item, |ui| {
            let bg_stroke =
                ui.visuals()
                    .widgets
                    .noninteractive
                    .bg_stroke;
            let (rect, _) =
                ui.allocate_exact_size(
                    Vec2::new(ui.available_width(), 1.0),
                    Sense::hover());
            ui.painter().hline(
                rect.x_range(),
                rect.center().y,
                bg_stroke);
        });
    }
}

#[derive(Setters)]
pub struct FlexCard {
    /// Direction of the inner layout.
    ///
    /// [`FlexDirection::Vertical`] by default.
    pub direction: FlexDirection,

    /// Inner margin of the card.
    ///
    /// `Margin::same(8)` for vertical and `Margin::same(4)` for horizontal
    /// by default.
    pub padding: Margin,

    /// Gap between items in the inner layout.
    ///
    /// `(6, 6)` for vertical and `(4, 4)` for horizontal by default.
    pub gap: Vec2,
}

impl FlexCard {
    pub const fn vertical() -> Self {
        Self {
            direction: FlexDirection::Vertical,
            padding: Margin::same(8),
            gap: Vec2::new(8.0, 8.0),
        }
    }

    pub const fn horizontal() -> Self {
        Self {
            direction: FlexDirection::Horizontal,
            padding: Margin::same(4),
            gap: Vec2::new(4.0, 4.0),
        }
    }
}

impl FlexCard {
    pub fn show<F>(self, flex: &mut FlexInstance, content: F)
    where
        F: FnOnce(&mut FlexInstance) {
        flex.add_ui(item(), |ui| self.show_ui(ui, content));
    }

    pub fn show_ui<F>(self, ui: &mut Ui, content: F)
    where
        F: FnOnce(&mut FlexInstance) {
        Frame::new()
            .fill(ui.visuals().faint_bg_color)
            .corner_radius(6.0)
            .inner_margin(self.padding)
            .show(ui, |ui| match self.direction {
                FlexDirection::Vertical =>
                    Flex::vertical()
                        .w_full()
                        .show(ui, |flex| {
                            flex.add_flex(
                                item(),
                                Flex::vertical()
                                    .w_full()
                                    .align_items(FlexAlign::Start)
                                    .gap(self.gap),
                                content);
                        }),
                FlexDirection::Horizontal =>
                    Flex::horizontal()
                        .w_full()
                        .gap(self.gap)
                        .show(ui, content),
            });
    }
}

#[derive(Setters)]
pub struct FlexActionButton<'a> {
    pub icon: &'a str,
    pub enabled: bool,
    pub selected: bool,
}

impl FlexActionButton<'_> {
    pub const fn new() -> Self {
        Self {
            icon: "",
            enabled: true,
            selected: false,
        }
    }
}

impl FlexWidget for FlexActionButton<'_> {
    type Response = Response;

    fn flex_ui(self, item: FlexItem, flex_instance: &mut FlexInstance) -> Self::Response {
        flex_instance.add_ui(
            item,
            Button::new(self.icon)
                .selected(self.selected)
                .pipe(|body| |ui: &mut Ui| {
                    ui.style_mut().spacing.button_padding = Vec2::ZERO;
                    ui.add(body)
                })
                .pipe(|body| |ui: &mut Ui| body(ui).on_hover_cursor(CursorIcon::PointingHand))
                .pipe(|body| |ui: &mut Ui| ui.add_enabled(self.enabled, body))
                .pipe(|body| |ui: &mut Ui| ui.add_sized([22.0, 22.0], body)))
            .inner
    }
}
