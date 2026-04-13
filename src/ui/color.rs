#![allow(dead_code, clippy::allow_attributes, reason = "color palette")]

use egui::Color32;

// — Surface palette (Zinc) —

const ZINC_900: Color32 = Color32::from_rgb(0x18, 0x18, 0x1B);
const ZINC_800: Color32 = Color32::from_rgb(0x27, 0x27, 0x2A);
const ZINC_700: Color32 = Color32::from_rgb(0x3F, 0x3F, 0x46);
const ZINC_400: Color32 = Color32::from_rgb(0xA1, 0xA1, 0xAA);

// — Extra palette (One Dark) —
pub const RED: Color32 =
    Color32::from_rgb(0xE0, 0x6C, 0x75);
pub const ORANGE: Color32 =
    Color32::from_rgb(0xD1, 0x9A, 0x66);
pub const YELLOW: Color32 =
    Color32::from_rgb(0xE5, 0xC0, 0x7B);
pub const GREEN: Color32 =
    Color32::from_rgb(0x98, 0xC3, 0x79);
pub const CYAN: Color32 =
    Color32::from_rgb(0x56, 0xB6, 0xC2);
pub const BLUE: Color32 =
    Color32::from_rgb(0x61, 0xAF, 0xEF);
pub const PURPLE: Color32 =
    Color32::from_rgb(0xC6, 0x78, 0xDD);

// — Semantic colors —

/// Main content background. Darker than the nav so the main canvas reads as
/// recessed.
pub const BACKGROUND: Color32 = ZINC_900;
/// Input background — text edits, scroll area gutters, etc.
pub const BACKGROUND_INPUT: Color32 = ZINC_900;
/// Alternative background. One step lighter than [`BACKGROUND`].
pub const BACKGROUND_ALT: Color32 = ZINC_800;
/// Card / row surface on the main canvas. One step lighter than [`BACKGROUND`].
pub const CARD: Color32 = ZINC_800;
/// Card surface under hover.
pub const CARD_HOVER: Color32 = ZINC_700;
/// Card surface while pressed / open.
pub const CARD_ACTIVE: Color32 = ZINC_700;
/// Subtle separator stroke — used for the few places (e.g. collapsing header
/// bottom border) where egui still draws a noninteractive stroke.
pub const BORDER:  Color32 = ZINC_700;
/// Default body text color. (zinc-400).
pub const FOREGROUND:   Color32 = ZINC_400;
/// Primary accent color for status badges, links, and selection highlights.
pub const PRIMARY: Color32 = BLUE;
