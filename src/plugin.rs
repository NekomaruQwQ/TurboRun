use std::collections::*;
use std::fs;
use std::path::Path;

use tap::prelude::*;

use anyhow::Context as _;
use itertools::Itertools as _;
use serde::Deserialize as _;
use smol_str::SmolStr;

use crate::data::*;

#[expect(clippy::type_complexity, reason = "collections")]
pub fn scan_plugins(plugin_dir: &Path)
 -> anyhow::Result<impl Iterator<Item = PluginPack>> {
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
        .sorted_by_key(|plugin| plugin.name.clone())
        .collect_vec()
        .pipe(|plugins| PluginPack {
            name: pack_name,
            path: path.into(),
            plugins,
        })
        .pipe(Ok)
}

pub fn collect_plugins<'a, I>(packs: I) -> PluginMap
 where
    I: IntoIterator<Item = &'a PluginPack>, {
    packs
        .into_iter()
        .flat_map(|pack| {
            pack.plugins
                .iter()
                .map(|plugin| {
                    ((
                        pack.name.clone(),
                        plugin.name.clone()),
                        plugin.clone())
                })
        })
        .collect()
}

pub fn apply_plugins(task: &Task, plugin_packs: &PluginPackMap)
 -> anyhow::Result<String> {
    let mut out = Vec::new();

    task.plugins
        .iter()
        .map(|item| item.pack.clone())
        .filter_map(|pack_name| {
            plugin_packs
                .get(&pack_name)
                .tap_none(|| {
                    log::error!(
                        "missing plugin pack {pack_name} for task \"{}\"",
                        &task.name);
                })
        })
        .map(|plugin| {
            plugin.path
                .to_str()
                .expect("@logicError invalid plugin_dir")
                .to_owned()
        })
        .map(|path| path.replace('\\', "/"))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .for_each(|path| out.push(format!("use \"{path}\"")));

    let mut i = 0;
    out.push(["let __closure_0 = { ", &task.command, " }"].join(""));
    i += 1;

    for item in &task.plugins {
        let mut line = format!(
            "{} {} $__closure_{}",
            item.pack,
            item.name,
            i - 1);
        for (key, value) in &item.args {
            line.push_str(&format!(" --{} \"{}\"", key, value));
        }
        for flag in &item.flags {
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
