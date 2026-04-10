//! Extra `impl` blocks for core data structures for TurboRun.

use super::*;

impl Task {
    pub fn empty() -> Self {
        Self {
            id: TaskId::random(),
            name: String::new(),
            command: String::new(),
            plugins: Vec::new(),
        }
    }

    pub fn example() -> Self {
        Self {
            name: "Example Task".into(),
            command: "print \"Hello, TurboRun!\"".into(),
            plugins: vec![
                PluginInstance {
                    file_name: "base.nu".into(),
                    item_name: "time".into(),
                    args: [("unit".into(), "s".into())].into(),
                    flags: [].into(),
                }
            ],
            ..Self::empty()
        }
    }
}
