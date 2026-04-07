//! Core data structures for TurboRun, shared among the persistance layer,
//! the task execution engine and the UI.

mod task_id;
pub use task_id::TaskId;

use std::path::PathBuf;
use std::time::SystemTime;

use serde::*;

/// Represents a TurboRun configuration, loaded from and saved to a TOML file.
///
/// Fields in this struct and all nested structs use `skip_serializing_if` so
/// that empty or default values are omitted from the TOML output, keeping
/// the config file minimal.
/// The corresponding `#[serde(default)]` on each field ensures that missing
/// sections are filled in on load.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[derive(Default)]
#[derive(Deserialize, Serialize)]
pub struct Config {
    pub tasks: Vec<Task>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[derive(Deserialize, Serialize)]
pub struct Task {
    /// The stable ID of this task.
    ///
    /// The ID is generated randomly when a new task is created, and should be
    /// stable across renames and other changes to the task.
    pub id: TaskId,

    /// The name of this task, e.g. "Print Hello".
    pub name: String,

    /// The command to execute for this task, e.g. "print hello".
    pub command: String,

    /// Plugins to load for this task.
    ///
    /// Plugins are referenced by their relative path from the plugins directory
    /// without extension.
    ///
    /// Plugins are applied in the order they are listed, i.e. the first plugin
    /// is the innermost wrapper around the command, and the last plugin is the
    /// outermost.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub plugins: Vec<PluginInstance>,

    /// The last time this task was modified by the user, in milliseconds since
    /// the Unix epoch.
    pub last_modified: SystemTime,
}

impl Task {
    pub fn empty() -> Self {
        Self {
            id: TaskId::random(),
            name: String::new(),
            command: String::new(),
            plugins: Vec::new(),
            last_modified: SystemTime::now(),
        }
    }

    pub fn example() -> Self {
        Self {
            name: "Example Task".into(),
            command: "print \"Hello, TurboRun!\"".into(),
            plugins: vec![
                PluginInstance::new("timed")
                    .var("unit", "ms")
            ],
            ..Self::empty()
        }
    }
}

/// Represents a plugin that can be applied to a task's command to modify its
/// behavior.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Plugin {
    /// The name of this plugin, uniquely identifying it among all plugins.
    ///
    /// For external plugins loaded from disk, this is derived from the relative
    /// path of the plugin's source file from the plugins directory without
    /// extension.
    pub name: String,
    /// The full path to the plugin file, or [`None`] for built-in plugins.
    pub path: Option<PathBuf>,
    /// The source code of this plugin.
    pub source: String,
    /// The last modified time of this plugin in milliseconds since the Unix
    /// epoch.
    ///
    /// For built-in plugins, this field is always zero.
    pub last_modified: SystemTime,
}

/// Represents a specific instance of a plugin applied to a task, including
/// the variables to be substituted into the plugin's source code when applied.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[derive(Deserialize, Serialize)]
pub struct PluginInstance {
    pub name: String,
    pub vars: Vec<(String, String)>,
}

impl PluginInstance {
    pub fn new(name: &str) -> Self {
        Self { name: name.into(), vars: Vec::new() }
    }

    pub fn var(mut self, name: &str, value: &str) -> Self {
        self.vars.push((name.into(), value.into()));
        self
    }
}
