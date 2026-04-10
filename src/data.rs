//! Core data structures for TurboRun, shared among the persistance layer,
//! the task execution engine and the UI.

mod task_id;
pub use task_id::TaskId;

mod ext;

use std::collections::BTreeMap;

use serde::*;

use crate::util::is_default;

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
    #[serde(default, skip_serializing_if = "is_default")]
    pub plugins: Vec<PluginInstance>,
}

/// Represents a plugin that can be applied to a task's command to modify its
/// behavior.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[derive(Deserialize, Serialize)]
pub struct Plugin {
    /// Name of the plugin file under the plugin directory, including the .nu
    /// extension. This field is not serialized.
    #[serde(skip)]
    pub file_name: String,

    /// Name of the custom command in the plugin file to be used as a plugin.
    #[serde(rename = "name")]
    pub item_name: String,

    /// A short description of this plugin's behavior and purpose.
    pub description: Option<String>,

    /// A list of args that this plugin accepts.
    #[serde(default)]
    #[serde(skip_serializing_if = "is_default")]
    pub args: Vec<PluginArg>,

    /// A list of flags that this plugin accepts.
    #[serde(default)]
    #[serde(skip_serializing_if = "is_default")]
    pub flags: Vec<PluginFlag>,
}

/// Represents a lookup table of plugins, indexed first by plugin file name and
/// then by item name for easy retrieval.
pub type PluginMap = BTreeMap<String, BTreeMap<String, Plugin>>;

/// Represents a flag that a Nushell custom command accepts.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[derive(Deserialize, Serialize)]
pub struct PluginFlag {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[derive(Deserialize, Serialize)]
pub struct PluginArg {
    pub name: String,
    pub description: Option<String>,

    #[serde(default)]
    #[serde(skip_serializing_if = "is_default")]
    pub required: bool,

    #[serde(default)]
    #[serde(skip_serializing_if = "is_default")]
    pub accepted_values: Vec<String>,
}

/// Represents a specific instance of a plugin applied to a task, including
/// the variables to be substituted into the plugin's source code when applied.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[derive(Deserialize, Serialize)]
pub struct PluginInstance {
    pub file_name: String,
    pub item_name: String,

    #[serde(default)]
    #[serde(skip_serializing_if = "is_default")]
    pub args: BTreeMap<String, String>,

    #[serde(default)]
    #[serde(skip_serializing_if = "is_default")]
    pub flags: Vec<String>,
}

impl PluginInstance {
    pub fn new(path: &str, name: &str) -> Self {
        Self {
            file_name: path.into(),
            item_name: name.into(),
            args: BTreeMap::new(),
            flags: Vec::new(),
        }
    }
}
