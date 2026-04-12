use std::fs;
use std::path::Path;

use serde::Deserialize as _;

use crate::prelude::*;
use crate::data::*;

pub fn load_plugin_pack_from_file(path: &Path) -> anyhow::Result<PluginPack> {
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

pub fn apply_plugins(task: &Task, plugin_packs: &PluginPackMap)
 -> anyhow::Result<String> {
    let mut out = Vec::new();

    // 1. Import plugin pack files.
    task.plugins
        .iter()
        .map(|item| {
            plugin_packs
                .get(&item.pack)
                .ok_or_else(|| anyhow::anyhow!("missing plugin pack {}", item.pack))
        })
        .collect::<anyhow::Result<Vec<_>>>()?
        .into_iter()
        .map(|plugin| {
            plugin.path
                .to_str()
                .expect("@logicError invalid plugin_dir")
                .replace('\\', "/")
        })
        .unique()
        .map(|path| format!("use \"{path}\""))
        .pipe(|lines| out.extend(lines));

    // 2. Build the closure chain for the task command and plugins.
    out.push(["let __closure_0 = { ", &task.command, " }"].join(""));

    let mut curr = 1;
    let mut prev = 0;
    #[expect(clippy::explicit_counter_loop, reason = "more readable with explicit counters")]
    for item in &task.plugins {
        format!("{} {} $__closure_{prev}", item.pack, item.name)
            .pipe(Some)
            .into_iter()
            .chain(item.args.iter().map(|(key, value)| format!("--{key} \"{value}\"")))
            .chain(item.flags.iter().map(|flag| format!("--{flag}")))
            .join(" ")
            .pipe(|line| format!("let __closure_{curr} = {{ {line} }}"))
            .pipe(|line| out.push(line));
        prev = curr;
        curr += 1;
    }

    // 3. Append the final command to run the last closure.
    out
        .tap_mut(|out| out.push(format!("do $__closure_{prev}")))
        .join("\n")
        .tap(|result| log::info!(">>>\n{result}\n<<<"))
        .pipe(Ok)
}
