use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use tap::prelude::*;

use anyhow::Context as _;
use itertools::Itertools as _;
use serde::Deserialize as _;
use smol_str::SmolStr;

use crate::data::*;

pub fn scan_plugins(plugin_dir: &Path) -> anyhow::Result<impl Iterator<Item = PluginPack>> {
    log::info!("scanning plugins at \"{}\" ...", plugin_dir.display());
    fs::read_dir(plugin_dir)
        .with_context(|| format!("fs::read_dir failed: {}", plugin_dir.display()))?
        .flatten()
        .map(|entry| entry.path())
        .filter_map(|path| {
            load_plugin_pack_from_file(&path)
                .tap_err(|err| log::error!("failed to load plugin pack at \"{}\": {err:?}", path.display()))
                .ok()?
                .pipe(Some)
        })
        .pipe(Ok)
}

fn load_plugin_pack_from_file(path: &Path) -> anyhow::Result<PluginPack> {
    use toml::Value as TomlValue;
    use toml::Table as TomlTable;

    let pack_name: SmolStr =
        path.file_stem()
            .expect("@logicError unexpected path")
            .to_str()
            .context("file name is not valid utf-8")?
            .into();
    if !(
        path.is_file() &&
        path.extension().is_some_and(|ext| ext == "nu")) {
        anyhow::bail!("not a .nu file");
    }

    fs::read_to_string(path)
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
        .ok_or_else(|| anyhow::anyhow!("invalid plugin metadata"))?
        .into_iter()
        .enumerate()
        .filter_map(|(index, toml)| {
            toml.pipe(Plugin::deserialize)
                .context("failed to deserialize plugin metadata")
                .tap_err(|err| log::error!("failed to load plugin #{index}: {err:?}"))
                .ok()
        })
        .map(|plugin| (plugin.name.clone(), plugin))
        .pipe(|plugins| PluginPack {
            path: path.to_path_buf(),
            name: pack_name,
            plugins: plugins.collect(),
        })
        .pipe(Ok)
}

pub fn apply_plugins(
    plugin_packs: &BTreeMap<SmolStr, PluginPack>,
    source: &str,
    plugins: &[PluginInstance])
 -> anyhow::Result<String> {
    let mut out = Vec::new();
    for inst in plugins {
        plugin_packs.get(&inst.pack)
            .map(|pack| pack.path.clone())
            .ok_or_else(|| anyhow::anyhow!("plugin pack \"{}\" not found", inst.pack))?
            .pipe(|path| path.to_str().expect("@logicError invalid plugin_dir").to_owned())
            .pipe(|path| path.replace('\\', "/"))
            .pipe(|path| format!("use \"{path}\""))
            .pipe(|line| out.push(line));
    }

    let mut i = 0;
    out.push(["let __closure_0 = { ", source, " }"].join(""));
    i += 1;

    for inst in plugins {
        let mut line = format!(
            "{} {} $__closure_{}",
            inst.pack,
            inst.name,
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
