use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use anyhow::Context as _;
use itertools::Itertools as _;
use tap::prelude::*;

use crate::util::*;
use crate::data::*;
use crate::plugin::*;
use crate::worker::*;

pub struct TaskEngine {
    plugins: HashMap<String, Plugin>,
    tasks: HashMap<TaskId, TaskWorker>,

    config_path: PathBuf,
    plugin_dir: PathBuf,
}

impl TaskEngine {
    pub fn new(
        config_path: PathBuf,
        plugin_dir: PathBuf) -> Self {
        Self {
            plugins: HashMap::new(),
            tasks: HashMap::new(),
            config_path,
            plugin_dir,
        }
    }

    pub const fn config_path(&self) -> &PathBuf {
        &self.config_path
    }

    pub const fn plugin_dir(&self) -> &PathBuf {
        &self.plugin_dir
    }

    pub const fn plugins(&self) -> &HashMap<String, Plugin> {
        &self.plugins
    }

    pub fn tasks(&self) -> impl Iterator<Item = &TaskWorker> {
        self.tasks
            .values()
            .sorted_by_key(|worker| &worker.task().name)
    }

    pub fn plugins_sorted(&self) -> impl Iterator<Item = &Plugin> {
        self.plugins
            .values()
            .sorted_by_key(|plugin| &plugin.name)
    }

    pub const fn tasks_mut(&mut self) -> &mut HashMap<TaskId, TaskWorker> {
        &mut self.tasks
    }

    pub fn load_config(&mut self) -> anyhow::Result<()> {
        if self.tasks.values().any(TaskWorker::is_running) {
            anyhow::bail!("cannot load config while tasks are running");
        }

        let config =
            std::fs::read_to_string(&self.config_path)
                .pipe(none_if_not_found)
                .context("fs::read_to_string failed")?
                .map(|toml| toml::from_str::<Config>(&toml))
                .transpose()
                .context("toml::from_str failed")?
                .unwrap_or_else(|| Config {
                    tasks: vec![Task::example()],
                });
        self.tasks =
            config.tasks
                .into_iter()
                .map(|task| (task.id, TaskWorker::new(task)))
                .collect();
        Ok(())
    }

    pub fn save_config(&self) -> anyhow::Result<()> {
        let tasks =
            self.tasks
                .values()
                .map(|worker| worker.task().clone())
                .collect();
        let config = Config {
            tasks,
        };

        toml::to_string_pretty(&config)
            .context("toml::to_string_pretty failed")?
            .pipe(|toml| fs::write(&self.config_path, &toml))
            .context("fs::write failed")?;
        Ok(())
    }

    pub fn scan_plugins(&mut self) -> anyhow::Result<()> {
        self.plugins =
            scan_plugins(&self.plugin_dir)?
                .into_iter()
                .map(|plugin| (plugin.name.clone(), plugin))
                .collect();
        Ok(())
    }
}
