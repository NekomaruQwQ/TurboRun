use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use tap::prelude::*;

use anyhow::Context as _;
use itertools::Itertools as _;
use serde::Deserialize as _;

use crate::data::*;

pub fn scan_plugins(plugin_dir: &Path) -> anyhow::Result<PluginMap> {
    log::info!("scanning plugins at \"{}\" ...", plugin_dir.display());
    fs::read_dir(plugin_dir)
        .with_context(|| format!("fs::read_dir failed: {}", plugin_dir.display()))?
        .into_iter()
        .flatten()
        .map(|entry| entry.path())
        .filter_map(|path| {
            check_plugin_file(&path)
                .tap_err(|err| log::warn!("skipping \"{}\": {err:?}", path.display()))
                .ok()
        })
        .filter_map(|file_name| {
            load_plugins_from_file(plugin_dir, &file_name)
                .tap_err(|err| log::error!("failed to load plugin file \"{file_name}\": {err:?}"))
                .ok()?
                .map(|plugin| (plugin.item_name.clone(), plugin))
                .collect::<BTreeMap<_, _>>()
                .pipe(|plugins| (file_name.clone(), plugins))
                .pipe(Some)
        })
        .collect::<BTreeMap<_, _>>()
        .pipe(Ok)
}

/// Checks if the given path is a valid plugin file (i.e. a .nu file) and
/// returns its file name if valid.
fn check_plugin_file(path: &Path) -> anyhow::Result<String> {
    let file_name =
        path.file_name()
            .expect("@logicError unexpected path");
    let file_name =
        file_name
            .to_str()
            .context("file name is not valid utf-8")?
            .to_owned();
    if path.is_file() && path.extension().is_some_and(|ext| ext == "nu") {
        file_name.pipe(Ok)
    } else {
        anyhow::bail!("not a .nu file");
    }
}

pub fn load_plugins_from_file(base: &Path, file_name: &str)
 -> anyhow::Result<impl Iterator<Item = Plugin>> {
    use toml::Value as TomlValue;
    use toml::Table as TomlTable;

    fs::read_to_string(base.join(file_name))
        .context("fs::read_to_string failed")?
        .pipe(|content| {
            content
                .lines()
                .map(str::trim)
                .filter_map(|line| line.strip_prefix("#?"))
                .join("\n")
        })
        .pipe(|content| toml::from_str::<TomlTable>(&content))
        .context("toml::from_str failed")?
        .pipe(|toml| {
            toml.get("plugins")
                .and_then(TomlValue::as_array)
                .cloned()
        })
        .ok_or(anyhow::anyhow!("invalid plugin metadata"))?
        .into_iter()
        .map(|value| parse_plugin(file_name, value))
        .filter_map(move |result| {
            result
                .tap_err(|err| log::error!("failed to parse plugin in \"{file_name}\": {err:?}"))
                .ok()
        })
        .pipe(Ok)
}

fn parse_plugin(file_name: &str, toml: toml::Value) -> anyhow::Result<Plugin> {
    toml.pipe(|toml| Plugin::deserialize(toml))
        .context("Plugin::deserialize failed")?
        .tap_mut(|plugin| plugin.file_name = file_name.into())
        .pipe(Ok)
}

pub fn apply_plugins(
    plugin_dir: &Path,
    source: &str,
    plugins: &[PluginInstance])
 -> anyhow::Result<String> {
    let mut out = Vec::new();
    for &PluginInstance { ref file_name, .. } in plugins.iter() {
        file_name
            .pipe(|file_name| plugin_dir.join(file_name))
            .pipe(|path| path.to_str().expect("@logicError invalid plugin_dir").to_owned())
            .pipe(|path| path.replace("\\", "/"))
            .pipe(|path| format!("use \"{path}\""))
            .pipe(|line| out.push(line));
    }

    let mut i = 0;
    out.push(["let __closure_0 = { ", source, " }"].join(""));
    i += 1;

    for inst in plugins {
        let mut line = format!(
            "{} {} $__closure_{}",
            inst.file_name
                .strip_suffix(".nu")
                .expect("@logicError invalid file_name"),
            inst.item_name,
            i - 1);
        for (key, value) in &inst.args {
            line.push_str(&format!(" --{} \"{}\"", key, value));
        }
        for flag in &inst.flags {
            line.push_str(&format!(" --{flag}"));
        }
        out.push(format!("let __closure_{i} = {{ {line} }}"));
        i += 1;
    }

    out.push(format!("do $__closure_{}", i - 1));
    out
        .join("\n")
        .tap(|result| log::info!(">>>\n{result}\n<<<"))
        .pipe(Ok)
}
