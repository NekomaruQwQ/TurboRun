//! Core data structures for TurboRun, shared among the persistance layer,
//! the task execution engine and the UI.

mod task_id;
pub use task_id::TaskId;

use std::collections::*;
use std::path::PathBuf;

use serde::*;
use smol_str::SmolStr;

use crate::util::*;

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

    /// The name of this task. This field is descriptive and does not affect the
    /// behavior of the task.
    ///
    /// Empty names are allowd and are rendered as "Unnamed Task" in the UI.
    ///
    /// Kept as [`String`] rather than [`SmolStr`] because it is rarely cloned.
    /// Keeping it as a [`String`] simplifies binding to [`egui::TextEdit`] for
    /// editing in the UI.
    pub name: String,

    /// The command to execute for this task.
    ///
    /// Kept as [`String`] rather than [`SmolStr`] because it is rarely cloned.
    /// Keeping it as a [`String`] simplifies binding to [`egui::TextEdit`] for
    /// editing in the UI.
    ///
    /// This field is not validated since it can contain arbitrary nushell code,
    /// and validating it would require parsing nushell syntax which is nontrivial
    /// and out of scope for our current validation needs. Instead, we rely on the
    /// fact that any syntax errors in the command will be caught by nushell when
    /// the task is run, and we can display those errors to the user at that time
    /// rather than at the time of editing.
    ///
    /// Note that empty commands are allowed since they can be useful for testing
    /// plugin behavior.
    pub command: String,

    /// Plugins to load for this task. See [`PluginInstance`] for details.
    ///
    /// Plugins are applied in the order they are listed, i.e. the first plugin
    /// is the innermost wrapper around the command, and the last plugin is the
    /// outermost.
    ///
    /// The first two [`PluginInstance`]s are stored inline in this struct to
    /// optimize for the common case of tasks with few plugins.
    #[serde(default, skip_serializing_if = "is_default")]
    pub plugins: Vec<PluginInstance>,
}

impl Task {
    pub fn label(&self) -> String {
        format!("Task #{} \"{}\"", self.id, self.name)
    }
}

/// Represents a plugin pack, which is a collection of related plugins defined
/// in a single Nushell module.
///
/// For simplicity and ease of use, we require that each plugin pack is backed
/// by a nushell file on disk. The name of the plugin pack is the Nushell module
/// name, which is derived from the file name of the nushell file, ignoring the
/// directory and the extension. This allows the TurboRun engine to simply use
/// Nushell import mechanics to load them.
///
/// This struct is not directly deserialized from the TOML metadata of the plugin
/// file. Instead, it is constructed from the file name and the list of plugins
/// parsed from the file so that parsing failures on individual plugins does not
/// cause the entire plugin pack to fail to load.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PluginPack {
    /// Path to the nushell file backing this plugin pack.
    pub path: PathBuf,

    /// Name of the plugin pack, derived from the file name of the nushell file.
    ///
    /// This is the Nushell module name when the plugin pack is imported, and
    /// must be unique across all plugin packs to avoid shadowing and import
    /// errors. The containing directory and the nushell extension are not included
    /// in the name.
    pub name: SmolStr,

    /// Plugins defined in this plugin pack, sorted by their command name.
    pub plugins: Vec<Plugin>,
}

/// Represents a custom Nushell command that can be applied to a task's command
/// to modify its behavior.
///
/// This struct is directly deserialized from the TOML metadata of the plugin
/// file except for the `pack` field, which is provided by the plugin loader.
///
/// This struct and its nested structs are not explicitly validated: any issues
/// with them will be caught by nushell when the task is run.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[derive(Default)]
#[derive(Deserialize, Serialize)]
pub struct Plugin {
    /// Name of the custom command in the plugin file to be used as a plugin.
    pub name: SmolStr,

    /// Optional description of this plugin's behavior and purpose.
    ///
    /// This does not affect the behavior of the plugin and is only used for
    /// display in the UI. Empty string means no description, and if so the
    /// field is omitted from the UI and the serialized TOML.
    #[serde(default, skip_serializing_if = "is_default")]
    pub description: String,

    /// A list of args that this plugin accepts.
    #[serde(default, skip_serializing_if = "is_default")]
    pub args: Vec<PluginArg>,

    /// A list of flags that this plugin accepts.
    #[serde(default, skip_serializing_if = "is_default")]
    pub flags: Vec<PluginFlag>,
}

/// Represents an argument that a nushell custom command accepts.
///
/// [`PluginArg`]s are by default required and can be marked optional by
/// setting the `optional` field to `true`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[derive(Default)]
#[derive(Deserialize, Serialize)]
pub struct PluginArg {
    /// Name of the argument.
    ///
    /// It must be in cabab-case as is required by nushell's syntax for named
    /// arguments, but we do not explicitly validate this and just let nushell
    /// report syntax errors if the plugin violates this requirement.
    pub name: SmolStr,

    /// Optional description of this argument and its purpose.
    ///
    /// This does not affect the behavior of the plugin and is only used for
    /// display in the UI. Empty string means no description, and if so the
    /// field is omitted from the UI and the serialized TOML.
    #[serde(default, skip_serializing_if = "is_default")]
    pub description: String,

    /// Whether this argument is optional or required. By default, all arguments
    /// are required.
    #[serde(default, skip_serializing_if = "is_default")]
    pub optional: bool,

    /// Lists accepted values for this argument, or omitted if arbitrary values
    /// are accepted.
    ///
    /// Note that `Some(vec![])` (an empty list of accepted values) is different
    /// from `None` and rejects all values.
    pub accepted_values: Option<Vec<SmolStr>>,
}

/// Represents an optional flag that a nushell custom command accepts.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[derive(Default)]
#[derive(Deserialize, Serialize)]
pub struct PluginFlag {
    /// Name of the flag.
    ///
    /// It must be in cabab-case as is required by nushell's syntax for named
    /// arguments, but we do not explicitly validate this and just let nushell
    /// report syntax errors if the plugin violates this requirement.
    pub name: SmolStr,

    /// Optional description of this flag and its purpose.
    ///
    /// This does not affect the behavior of the plugin and is only used for
    /// display in the UI. Empty string means no description, and if so the
    /// field is omitted from the UI and the serialized TOML.
    #[serde(default, skip_serializing_if = "is_default")]
    pub description: String,
}

/// Represents a specific instance of a plugin applied to a task, including
/// the variables to be substituted into the plugin's source code when applied.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[derive(Default)]
#[derive(Deserialize, Serialize)]
pub struct PluginInstance {
    /// Name of the plugin pack.
    pub pack: SmolStr,

    /// Name of the custom command in the plugin pack.
    pub name: SmolStr,

    /// Whether this plugin instance is enabled.
    ///
    /// This provides a convenient way to temporarily disable a plugin without
    /// having to remove it from the task.
    #[serde(default, skip_serializing_if = "is_default")]
    pub enabled: bool,

    /// Argument assignments for this plugin instance.
    #[serde(default, skip_serializing_if = "is_default")]
    pub args: BTreeMap<SmolStr, SmolStr>,

    /// Flags enabled for this plugin instance.
    #[serde(default, skip_serializing_if = "is_default")]
    pub flags: Vec<SmolStr>,
}

pub type PluginKey = (SmolStr, SmolStr);
pub type PluginMap = BTreeMap<PluginKey, Plugin>;
pub type PluginPackMap = BTreeMap<SmolStr, PluginPack>;

impl PluginInstance {
    pub fn plugin(&self) -> PluginKey {
        (self.pack.clone(), self.name.clone())
    }
}

impl garde::Validate for PluginInstance {
    type Context = PluginMap;

    /// Validates this plugin instance against the loaded plugins in the engine,
    /// which are provided as the validation context.
    fn validate_into(
        &self,
        ctx: &Self::Context,
        parent: &mut dyn FnMut() -> garde::Path,
        report: &mut garde::Report) {
        // Look up the plugin by composite key. A miss is reported against
        // `pack` (the user's first point of failure) and short-circuits the
        // remaining checks: without a plugin spec we can't validate args or
        // flags, and piling on speculative errors would just be noise.
        let Some(plugin) = ctx.get(&self.plugin()) else {
            report.append(
                parent().join("pack"),
                garde::Error::new(
                    format!("unknown plugin \"{}::{}\"", self.pack, self.name)));
            return;
        };

        // Required args present.
        for arg in &plugin.args {
            if !arg.optional && !self.args.contains_key(&arg.name) {
                report.append(
                    parent().join("args"),
                    garde::Error::new(format!(
                        "missing required argument \"{}\"",
                        arg.name)));
            }
        }

        // Arg key/value sanity. We build the accepted-key set once so unknown
        // keys are O(1) instead of O(args²).
        let known_args: HashSet<&str> =
            plugin.args.iter().map(|arg| arg.name.as_str()).collect();
        for (key, value) in &self.args {
            if !known_args.contains(key.as_str()) {
                report.append(
                    parent().join("args"),
                    garde::Error::new(format!("unknown argument \"{key}\"")));
                continue;
            }
            // Safe to find here — `known_args` proves the entry exists.
            if let Some(arg) = plugin.args.iter().find(|arg| arg.name == *key)
                && let Some(ref accepted) = arg.accepted_values
                && !accepted.contains(value) {
                report.append(
                    parent().join("args"),
                    garde::Error::new(format!(
                        "invalid value \"{value}\" for argument \"{key}\"")));
            }
        }

        // Flags must exist on the plugin spec — catches typos that previously
        // would have slipped through validation and surfaced as nushell parse
        // errors at runtime.
        let known_flags: HashSet<&str> =
            plugin.flags.iter().map(|flag| flag.name.as_str()).collect();
        for flag in &self.flags {
            if !known_flags.contains(flag.as_str()) {
                report.append(
                    parent().join("flags"),
                    garde::Error::new(format!("unknown flag \"{flag}\"")));
            }
        }
    }
}
