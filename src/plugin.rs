use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;
use std::time::SystemTime;

use anyhow::Context as _;
use tap::Pipe as _;

use crate::core::*;

fn none_if_not_found<T>(result: io::Result<T>) -> io::Result<Option<T>> {
    match result {
        Ok(value) =>
            Ok(Some(value)),
        Err(err) if err.kind() == io::ErrorKind::NotFound =>
            Ok(None),
        Err(err) =>
            Err(err),
    }
}

pub fn scan_plugins(dir: &Path) -> anyhow::Result<Vec<Plugin>> {
    fs::read_dir(dir)
        .context("fs::read_dir failed")?
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
        } else if file_name.ends_with(".meta.toml") {
            Ok(Vec::new())
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
pub fn load_plugin_from_file(plugin_dir: &Path, source_path: &Path)
 -> anyhow::Result<Plugin> {
    assert!(source_path.starts_with(plugin_dir));
    assert!(source_path.is_file());
    assert!(source_path.extension().unwrap_or_default() == "nu");

    let metadata_path =
        source_path.with_extension("meta.toml");
    let source =
        fs::read_to_string(source_path)
            .context("fs::read_to_string failed")?;
    let source_last_modified =
        fs::metadata(source_path)
            .and_then(|metadata| metadata.modified())
            .context("fs::metadata failed")?;
    let metadata =
        fs::read_to_string(&metadata_path)
            .pipe(none_if_not_found)
            .context("fs::read_to_string failed")?
            .map(|content| toml::from_str(&content))
            .transpose()
            .context("toml::from_str failed")?
            .unwrap_or_default();
    let metadata_last_modified =
        fs::metadata(&metadata_path)
            .pipe(none_if_not_found)
            .context("fs::metadata failed")?
            .map(|metadata| metadata.modified())
            .transpose()
            .context("fs::metadata failed")?
            .unwrap_or(SystemTime::UNIX_EPOCH);
    let last_modified =
        SystemTime::max(
            source_last_modified,
            metadata_last_modified)
            .duration_since(SystemTime::UNIX_EPOCH)
            .context("SystemTime::duration_since failed")?
            .as_millis() as u64;
    let plugin_id =
        source_path
            .strip_prefix(plugin_dir)
            .context("Path::strip_prefix failed")?
            .with_extension("")
            .to_string_lossy()
            .replace('\\', "/");
    Ok(Plugin {
        id: plugin_id,
        metadata,
        metadata_path,
        source,
        source_path: source_path.to_owned(),
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
                .get(&inst.id)
                .with_context(|| format!("plugin not found: {}", inst.id))?
                .source
                .clone();
        for (key, value) in &inst.variables {
            plugin = plugin.replace(&["{{", key, "}}"].concat(), value);
        }

        out = plugin.replace("{{COMMAND}}", &out);
    }
    Ok(out)
}
