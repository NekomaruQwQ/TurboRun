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
///
/// The corresponding `#[serde(default)]` on each field ensures that missing
/// sections are filled in on load.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[derive(Default)]
#[derive(Deserialize, Serialize)]
pub struct Config {
    /// List of tasks defined in this config.
    #[serde(default, skip_serializing_if = "is_default")]
    pub tasks: Vec<Task>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[derive(Default)]
#[derive(Deserialize, Serialize)]
pub struct Task {
    /// The stable ID of this task, uniquely identifying the task.
    ///
    /// The ID is generated randomly when a new task is created and stays stable
    /// across renames and other changes to the task.
    pub id: TaskId,

    /// The name of this task.
    pub name: String,

    /// The command to execute for this task.
    pub command: String,

    /// Plugins to load for this task. See [`PluginInstance`] for details.
    ///
    /// Plugins are applied in the order they are listed, i.e. the first plugin
    /// is the innermost wrapper around the command, and the last plugin is the
    /// outermost.
    #[serde(default, skip_serializing_if = "is_default")]
    pub plugins: Vec<PluginInstance>,
}

/// Represents a plugin pack, which is a collection of related plugins defined
/// in a single .nu file.
///
/// This struct is not directly deserialized from the TOML metadata of the plugin
/// file. Instead, it is constructed from the file name and the list of plugins
/// parsed from the file so that parsing failures on individual plugins does not
/// cause the entire plugin pack to fail to load.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PluginPack {
    /// Name of the plugin pack, derived from the file name of the .nu file,
    /// including the .nu extension. This field is not serialized.
    pub name: String,

    /// Plugins defined in this plugin pack, indexed by their name.
    pub plugins: BTreeMap<String, Plugin>,
}


/// Represents a custom Nushell command that can be applied to a task's command
/// to modify its behavior.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[derive(Deserialize, Serialize)]
pub struct Plugin {
    /// Name of the plugin pack that contains this plugin, derived from the file
    /// name of the .nu file. This field is not serialized.
    #[serde(skip)]
    pub pack: String,

    /// Name of the custom command in the plugin file to be used as a plugin.
    pub name: String,

    /// Optional description of this plugin's behavior and purpose.
    pub description: Option<String>,

    /// A list of args that this plugin accepts.
    #[serde(default, skip_serializing_if = "is_default")]
    pub args: Vec<PluginArg>,

    /// A list of flags that this plugin accepts.
    #[serde(default, skip_serializing_if = "is_default")]
    pub flags: Vec<PluginFlag>,
}

/// Represents an argument that a Nushell custom command accepts.
///
/// [`PluginArg`]s are by default required and can be marked optional by
/// setting the `optional` field to `true`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[derive(Deserialize, Serialize)]
pub struct PluginArg {
    /// Name of the argument. Must be in cabab-case as is required by Nushell's
    /// syntax for named arguments.
    pub name: String,

    /// Optional description of this argument and its purpose.
    pub description: Option<String>,

    /// Whether this argument is optional or required. By default, all arguments
    /// are required.
    #[serde(default, skip_serializing_if = "is_default")]
    pub optional: bool,

    /// Lists accepted values for this argument, or omitted if arbitrary values
    /// are accepted.
    ///
    /// Note that `Some(vec![])` (an empty list of accepted values) is different
    /// from `None` and rejects all values.
    pub accepted_values: Option<Vec<String>>,
}

/// Represents an optional flag that a Nushell custom command accepts.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[derive(Deserialize, Serialize)]
pub struct PluginFlag {
    /// Name of the flag. Must be in cabab-case as is required by Nushell's syntax
    /// for boolean arguments.
    pub name: String,

    /// Optional description of this flag and its purpose.
    pub description: Option<String>,
}

/// Represents a specific instance of a plugin applied to a task, including
/// the variables to be substituted into the plugin's source code when applied.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[derive(Deserialize, Serialize)]
pub struct PluginInstance {
    /// Name of the plugin pack derived from the file name of the .nu file,
    /// including the .nu extension.
    pub pack: String,

    /// Name of the custom command in the plugin file to be used as a plugin.
    pub name: String,

    /// Whether this plugin instance is enabled.
    ///
    /// This provides a convenient way to temporarily disable a plugin without
    /// having to remove it from the task.
    #[serde(default, skip_serializing_if = "is_default")]
    pub enabled: bool,

    /// Argument assignments for this plugin instance.
    #[serde(default, skip_serializing_if = "is_default")]
    pub args: BTreeMap<String, String>,

    /// Flags enabled for this plugin instance.
    #[serde(default, skip_serializing_if = "is_default")]
    pub flags: Vec<String>,
}

impl PluginInstance {
    pub fn new<S: Into<String>>(pack: S, name: S) -> Self {
        Self {
            pack: pack.into(),
            name: name.into(),
            enabled: true,
            args: BTreeMap::new(),
            flags: Vec::new(),
        }
    }
}
