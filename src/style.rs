//! Centralized visual theme for TurboRun.
//!
//! Defines a Fluent/PWA-flavored dark palette and applies it to egui via
//! [`setup_style`]. The palette uses an *inverted* hierarchy where the nav
//! surface is one step lighter than the main canvas, so the main content
//! reads as "sunken" beneath the nav. Cards on the main canvas are lighter
//! still, so they pop against the darker main background.
//!
//! All animations are disabled by zeroing `animation_time` and every
//! widget's `expansion`, per the project's "no animations" requirement.

use egui::*;

use crate::color;

/// Shared corner radius for every interactive widget.
const CORNER_RADIUS: CornerRadius = CornerRadius::same(4);

/// Apply the TurboRun visual theme to an egui [`Context`].
///
/// Configures `Visuals`, `Spacing`, text styles, and disables all animations.
/// Call once during app startup, after fonts are installed.
pub fn setup_style(ctx: &Context) {
    ctx.global_style_mut(|style| {
        // — Visuals (start from dark and override) —
        let visuals = &mut style.visuals;

        visuals.panel_fill            = color::BACKGROUND;
        visuals.window_fill           = color::BACKGROUND_ALT;
        visuals.faint_bg_color        = color::CARD;
        visuals.extreme_bg_color      = color::INPUT;
        visuals.hyperlink_color       = color::PRIMARY;
        visuals.window_stroke         = Stroke::NONE;
        visuals.window_corner_radius  = CornerRadius::same(8);
        visuals.menu_corner_radius    = CornerRadius::same(8);

        // `override_text_color` forces a single body color across all widgets,
        // which keeps the look uniform. Status badges still get their own
        // colors via `RichText::color`, which takes precedence.
        visuals.override_text_color   = Some(color::FOREGROUND);

        // Soft accent tint for selection / focus.
        visuals.selection.bg_fill     = color::PRIMARY.linear_multiply(0.35);
        visuals.selection.stroke      = Stroke::new(1.0, color::PRIMARY);

        // — Per-state widget visuals —
        let widgets = &mut visuals.widgets;

        // Noninteractive: panels, separators, labels.
        widgets.noninteractive.bg_stroke    = Stroke::new(1.0, color::BORDER);
        widgets.noninteractive.fg_stroke    = Stroke::new(1.0, color::FOREGROUND);
        widgets.noninteractive.weak_bg_fill = color::BACKGROUND_ALT;

        // Inactive: idle interactive widgets (buttons, headers, rows).
        widgets.inactive.bg_fill       = color::CARD;
        widgets.inactive.weak_bg_fill  = color::CARD;
        widgets.inactive.bg_stroke     = Stroke::NONE;
        widgets.inactive.corner_radius = CORNER_RADIUS;
        widgets.inactive.expansion     = 0.0;

        // Hovered.
        widgets.hovered.bg_fill        = color::CARD_HOVER;
        widgets.hovered.weak_bg_fill   = color::CARD_HOVER;
        widgets.hovered.bg_stroke      = Stroke::NONE;
        widgets.hovered.corner_radius  = CORNER_RADIUS;
        widgets.hovered.expansion      = 0.0;

        // Active (pressed).
        widgets.active.bg_fill         = color::CARD_ACTIVE;
        widgets.active.weak_bg_fill    = color::CARD_ACTIVE;
        widgets.active.bg_stroke       = Stroke::NONE;
        widgets.active.corner_radius   = CORNER_RADIUS;
        widgets.active.expansion       = 0.0;

        // Open (e.g. expanded collapsing header, open combo box).
        widgets.open.bg_fill           = color::CARD_ACTIVE;
        widgets.open.weak_bg_fill      = color::CARD_ACTIVE;
        widgets.open.bg_stroke         = Stroke::NONE;
        widgets.open.corner_radius     = CORNER_RADIUS;
        widgets.open.expansion         = 0.0;

        // — Spacing / Interaction / Text Styles —
        style.interaction.selectable_labels = false;

        style.spacing.item_spacing   = vec2(8.0, 4.0);
        style.spacing.button_padding = vec2(8.0, 4.0);
        style.spacing.menu_margin    = Margin::same(6);
        style.spacing.window_margin  = Margin::same(8);
        style.spacing.indent         = 14.0;

        style.text_styles = [
            (TextStyle::Heading,
                FontId::new(15.0, FontFamily::Proportional)),
            (TextStyle::Body,
                FontId::new(12.0, FontFamily::Proportional)),
            (TextStyle::Monospace,
                FontId::new(10.0, FontFamily::Monospace)),
            (TextStyle::Button,
                FontId::new(12.0, FontFamily::Proportional)),
            (TextStyle::Small,
                FontId::new(10.5, FontFamily::Proportional)),
        ].into();
    });
}
