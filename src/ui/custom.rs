use egui::*;

use crate::engine::TaskStatus;
use super::color;

pub fn code_block(source: &str) -> RichText {
    RichText::new(source.trim_ascii_end())
        .monospace()
        .line_height(Some(15.0))
}

pub fn task_status_label(status: TaskStatus) -> RichText {
    match status {
        TaskStatus::Invalid =>
            RichText::new("Invalid").color(color::ORANGE),
        TaskStatus::None =>
            RichText::new(""),
        TaskStatus::Running =>
            RichText::new("Running").color(color::BLUE),
        TaskStatus::Stopped =>
            RichText::new("Stopped").weak(),
        TaskStatus::Success =>
            RichText::new("Success").color(color::GREEN),
        TaskStatus::Failure =>
            RichText::new("Failed").color(color::RED),
    }
}
