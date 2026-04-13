use tap::prelude::*;

use egui::*;
use egui_flex::*;
use derive_setters::Setters;

pub struct FlexSpace(pub f32);

impl FlexSpace {
    pub fn fill(flex: &mut FlexInstance) {
        flex.add(item().grow(1.0), Self(0.0));
    }
}

impl FlexWidget for FlexSpace {
    type Response = ();

    fn flex_ui(self, item: FlexItem, flex_instance: &mut FlexInstance) -> Self::Response {
        flex_instance
            .pipe(|flex| (flex.ui(), flex.is_vertical()))
            .pipe(|(ui, is_vertical)| {
                if is_vertical {
                    Vec2::new(ui.available_width(), self.0)
                } else {
                    Vec2::new(self.0, ui.available_height())
                }
            })
            .pipe(|size| move |ui: &mut Ui| ui.allocate_space(size))
            .pipe(|body| flex_instance.add_ui(item, body));
    }
}

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

#[derive(Default)]
#[derive(Setters)]
pub struct FlexCard<'a> {
    pub item: FlexItem<'a>,

    /// Inner margin of the card. If `None`, a default margin of 4.0 points on
    /// all sides is used.
    #[setters(strip_option)]
    pub padding: Option<Margin>,

    /// Whether the card should stretch items to fill the available horizontal
    /// space.
    ///
    /// If the content contains an embedded [`Flex`] and would like to use up
    /// full horizontal space of the card, this must be set to `true` together
    /// with [`Flex::w_full`] on the embedded [`Flex`].
    #[setters(bool)]
    pub stretch: bool,
}

impl FlexCard<'_> {
    pub fn show<F>(
        self,
        flex: &mut FlexInstance,
        content: F)
     where F: FnOnce(&mut FlexInstance) {
        let card_color =
            flex.ui()
                .visuals()
                .faint_bg_color;
        flex.add_ui(self.item, |ui| {
            Frame::new()
                .fill(card_color)
                .corner_radius(6.0)
                .inner_margin(self.padding.unwrap_or(Margin::same(4)))
                .show(ui, |ui| {
                    Flex::vertical().w_full().show(ui, |flex| {
                        if self.stretch {
                            content(flex);
                        } else {
                            flex.add_flex(
                                item(),
                                Flex::vertical()
                                    .w_full()
                                    .align_items(FlexAlign::Start)
                                    .gap((6.0, 6.0).into()),
                                content);
                        }
                    });
                });
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
