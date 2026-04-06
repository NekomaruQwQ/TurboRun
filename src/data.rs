//! Core data structures for TurboRun, shared among the persistance layer,
//! the task execution engine and the UI.

mod task_id;
pub use task_id::TaskId;

use std::collections::HashMap;
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
#[derive(Debug, Clone)]
#[derive(Default)]
#[derive(Deserialize, Serialize)]
pub struct Config {
    pub tasks: Vec<Task>,
}

#[derive(Debug, Clone)]
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
    pub plugins: Vec<PluginInstance>,

    /// The last time this task was modified by the user, in milliseconds since
    /// the Unix epoch.
    pub last_modified: u64,
}

/// Represents a plugin that can be applied to a task's command to modify its
/// behavior.
#[derive(Debug, Clone)]
pub struct Plugin {
    /// The unique identifier of this plugin, which is the relative path from
    /// the plugins directory without extension.
    pub id: String,
    /// The metadata of this plugin, loaded from the plugin's metadata file
    /// (.meta.toml).
    pub metadata: PluginMetadata,
    /// The precalculated path to the metadata file of this plugin for reloading.
    pub metadata_path: PathBuf,
    /// The source code of this plugin, loaded from the plugin's source file.
    pub source: String,
    /// The precalculated path to the source file of this plugin for reloading.
    pub source_path: PathBuf,
    /// The last modified time of this plugin in milliseconds since the Unix epoch
    /// used for change detection and reloading.
    pub last_modified: u64,
}

/// Represents a metadata file for a plugin.
#[derive(Debug, Clone)]
#[derive(Default)]
#[derive(Deserialize, Serialize)]
pub struct PluginMetadata {
    pub name: Option<String>,
    pub description: Option<String>,
}

/// Represents a specific instance of a plugin applied to a task, including
/// the variables to be substituted into the plugin's source code when applied.
#[derive(Debug, Clone)]
#[derive(Deserialize, Serialize)]
pub struct PluginInstance {
    pub id: String,
    pub variables: HashMap<String, String>,
}
