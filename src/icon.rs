
//! Font Awesome glyphs from UbuntuMono Nerd Font, available app-wide via the
//! font fallback installed in [`setup_fonts`]. Names mirror Nerd Fonts'
//! `nf-fa-*` cheatsheet entries; codepoints sit in the BMP Private Use Area
//! (U+F000–U+F8FF) so each constant is a single 3-byte UTF-8 sequence and
//! can be passed anywhere a `&str` label is expected.

#![allow(dead_code, clippy::allow_attributes, reason = "unused icons")]

pub const BACK:    &str = "\u{F060}"; // nf-fa-arrow_left
pub const FORWARD: &str = "\u{F061}"; // nf-fa-arrow_right
pub const UP:      &str = "\u{F062}"; // nf-fa-arrow_up
pub const DOWN:    &str = "\u{F063}"; // nf-fa-arrow_down

pub const HOME:    &str = "\u{F015}"; // nf-fa-home
pub const PLUGIN:  &str = "\u{F12E}"; // nf-fa-puzzle_piece

pub const PLAY:    &str = "\u{F04B}"; // nf-fa-play
pub const STOP:    &str = "\u{F04D}"; // nf-fa-stop
pub const EDIT:    &str = "\u{F044}"; // nf-fa-edit
pub const SAVE:    &str = "\u{F0C7}"; // nf-fa-floppy_o

pub const CREATE:  &str = "\u{F067}"; // nf-fa-plus
pub const DELETE:  &str = "\u{F1F8}"; // nf-fa-trash

pub const CLOSE:   &str = "\u{F00D}"; // nf-fa-times
pub const REFRESH: &str = "\u{F021}"; // nf-fa-refresh
