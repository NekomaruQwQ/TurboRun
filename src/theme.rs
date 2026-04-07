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

// — Surface palette (OneDark-aligned, dark → light) —
//
// `Color32::from_rgb` is `const`, so these can live as crate constants and be
// referenced from other modules (e.g. `app.rs` for the nav panel fill).

const ZINC_950: Color32 = Color32::from_rgb(0x09, 0x09, 0x0B);
const ZINC_900: Color32 = Color32::from_rgb(0x18, 0x18, 0x1B);
const ZINC_800: Color32 = Color32::from_rgb(0x27, 0x27, 0x2A);
const ZINC_700: Color32 = Color32::from_rgb(0x3F, 0x3F, 0x46);
const ZINC_400: Color32 = Color32::from_rgb(0xA1, 0xA1, 0xAA);

/// Deepest sunken surface — text edits, scroll area gutters, etc.
pub const COLOR_INPUT: Color32 = ZINC_950;
/// Main content background. Darker than the nav so the main canvas reads as
/// recessed.
pub const COLOR_BACKGROUND: Color32 = ZINC_900;
/// Alternative background. One step lighter than [`COLOR_BACKGROUND`].
pub const COLOR_BACKGROUND_ALT: Color32 = ZINC_800;
/// Card / row surface on the main canvas. One step lighter than [`COLOR_BACKGROUND`].
pub const COLOR_CARD: Color32 = ZINC_800;
/// Card surface under hover.
pub const COLOR_CARD_HOVER: Color32 = ZINC_700;
/// Card surface while pressed / open.
pub const COLOR_CARD_ACTIVE: Color32 = ZINC_700;
/// Subtle separator stroke — used for the few places (e.g. collapsing header
/// bottom border) where egui still draws a noninteractive stroke.
pub const COLOR_BORDER:  Color32 = ZINC_700;
/// Default body text color. (zinc-400).
pub const COLOR_FOREGROUND:   Color32 = ZINC_400;

/// Primary accent color for status badges, links, and selection highlights.
pub const COLOR_PRIMARY: Color32 = crate::color::BLUE;

/// Shared corner radius for every interactive widget.
const CORNER_RADIUS: CornerRadius = CornerRadius::same(4);

/// Apply the TurboRun visual theme to an egui [`Context`].
///
/// Configures `Visuals`, `Spacing`, text styles, and disables all animations.
/// Call once during app startup, after fonts are installed.
pub fn setup_style(ctx: &Context) {
    ctx.global_style_mut(|style| {
        // — Visuals (start from dark and override) —
        let v = &mut style.visuals;

        v.panel_fill            = COLOR_BACKGROUND;
        v.window_fill           = COLOR_BACKGROUND_ALT;
        v.faint_bg_color        = COLOR_CARD;
        v.extreme_bg_color      = COLOR_INPUT;
        v.hyperlink_color       = COLOR_PRIMARY;
        v.window_stroke         = Stroke::NONE;
        v.window_corner_radius  = CornerRadius::same(8);
        v.menu_corner_radius    = CornerRadius::same(8);
        // `override_text_color` forces a single body color across all widgets,
        // which keeps the look uniform. Status badges still get their own
        // colors via `RichText::color`, which takes precedence.
        v.override_text_color   = Some(COLOR_FOREGROUND);

        // Soft accent tint for selection / focus.
        v.selection.bg_fill     = COLOR_PRIMARY.linear_multiply(0.35);
        v.selection.stroke      = Stroke::new(1.0, COLOR_PRIMARY);

        // — Per-state widget visuals —
        let w = &mut v.widgets;

        // Noninteractive: panels, separators, labels.
        w.noninteractive.bg_stroke    = Stroke::new(1.0, COLOR_BORDER);
        w.noninteractive.fg_stroke    = Stroke::new(1.0, COLOR_FOREGROUND);
        w.noninteractive.weak_bg_fill = COLOR_BACKGROUND_ALT;

        // Inactive: idle interactive widgets (buttons, headers, rows).
        w.inactive.bg_fill       = COLOR_CARD;
        w.inactive.weak_bg_fill  = COLOR_CARD;
        w.inactive.bg_stroke     = Stroke::NONE;
        w.inactive.corner_radius = CORNER_RADIUS;
        w.inactive.expansion     = 0.0;

        // Hovered.
        w.hovered.bg_fill        = COLOR_CARD_HOVER;
        w.hovered.weak_bg_fill   = COLOR_CARD_HOVER;
        w.hovered.bg_stroke      = Stroke::NONE;
        w.hovered.corner_radius  = CORNER_RADIUS;
        w.hovered.expansion      = 0.0;

        // Active (pressed).
        w.active.bg_fill         = COLOR_CARD_ACTIVE;
        w.active.weak_bg_fill    = COLOR_CARD_ACTIVE;
        w.active.bg_stroke       = Stroke::NONE;
        w.active.corner_radius   = CORNER_RADIUS;
        w.active.expansion       = 0.0;

        // Open (e.g. expanded collapsing header, open combo box).
        w.open.bg_fill           = COLOR_CARD_ACTIVE;
        w.open.weak_bg_fill      = COLOR_CARD_ACTIVE;
        w.open.bg_stroke         = Stroke::NONE;
        w.open.corner_radius     = CORNER_RADIUS;
        w.open.expansion         = 0.0;

        // — Style / spacing / interaction —
        //
        // `animation_time = 0.0` is the global kill switch for egui's hover
        // fades; combined with `expansion = 0.0` on every widget state, this
        // removes all transitions and "bump" effects.
        style.animation_time = 0.0;
        style.interaction.selectable_labels = false;

        style.spacing.item_spacing   = vec2(8.0, 4.0);
        style.spacing.button_padding = vec2(8.0, 4.0);
        style.spacing.menu_margin    = Margin::same(6);
        style.spacing.window_margin  = Margin::same(8);
        style.spacing.indent         = 14.0;

        // Text styles — preserved verbatim from the previous inline setup.
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
