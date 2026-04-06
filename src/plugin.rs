use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::SystemTime;

use anyhow::Context as _;
use tap::Pipe as _;

use crate::util::*;
use crate::data::*;

pub fn scan_plugins(dir: &Path) -> anyhow::Result<Vec<Plugin>> {
    fs::read_dir(dir)
        .pipe(none_if_not_found)
        .context("fs::read_dir failed")?
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .flat_map(|entry| {
            scan_plugins_from_dir_entry(dir, &entry)
                .unwrap_or_else(|err| {
                    log::warn!(
                        "failed to scan plugins at {}: {err:?}",
                        entry.path().display());
                    Vec::new()
                })
        })
        .collect::<Vec<_>>()
        .pipe(Ok)
}

/// Loads a plugin at the given path.
fn scan_plugins_from_dir_entry(
    plugin_dir: &Path,
    entry: &fs::DirEntry)
 -> anyhow::Result<Vec<Plugin>> {
    let path = entry.path();
    let path_display = path.display();
    let file_type =
        entry
            .file_type()
            .context("fs::file_type failed")?;

    if !file_type.is_dir() {
        let file_name =
            path.file_name()
                .unwrap_or_default()
                .to_string_lossy();
        if file_name.ends_with(".nu") {
            load_plugin_from_file(plugin_dir, &path)?
                .pipe(|plugin| vec![plugin])
                .pipe(Ok)
        } else {
            log::warn!("skipping non-plugin file at {path_display}");
            Ok(Vec::new())
        }
    } else {
        scan_plugins(&path)
    }
}

/// Loads a plugin at the given path.
#[expect(
    clippy::missing_assert_message,
    clippy::panic_in_result_fn,
    reason = "precondition check")]
pub fn load_plugin_from_file(base: &Path, path: &Path)
 -> anyhow::Result<Plugin> {
    assert!(path.starts_with(base));
    assert!(path.is_file());
    assert!(path.extension().unwrap_or_default() == "nu");

    let name =
        path
            .strip_prefix(base)
            .context("Path::strip_prefix failed")?
            .with_extension("")
            .to_string_lossy()
            .replace('\\', "/");
    let source =
        fs::read_to_string(path)
            .context("fs::read_to_string failed")?;
    let last_modified =
        fs::metadata(path)
            .and_then(|metadata| metadata.modified())
            .context("fs::metadata failed")?;
    log::info!("loaded plugin {name}");
    Ok(Plugin {
        name,
        source,
        path: Some(path.to_owned()),
        last_modified,
    })
}

#[expect(clippy::iter_over_hash_type, reason = "replacement order does not matter")]
pub fn apply_plugins(
    plugin_map: &HashMap<String, Plugin>,
    plugin_vec: &[PluginInstance],
    source: &str)
 -> anyhow::Result<String> {
    let mut out = source.to_owned();
    for inst in plugin_vec {
        let mut plugin =
            plugin_map
                .get(&inst.name)
                .with_context(|| format!("plugin not found: {}", inst.name))?
                .source
                .clone();
        for (key, value) in &inst.vars {
            plugin = plugin.replace(&["{{", key, "}}"].concat(), value);
        }

        out = plugin.replace("{{command}}", &out);
    }
    Ok(out)
}
