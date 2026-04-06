use std::collections::HashMap;
use std::fs;
use std::path::Path;

use anyhow::Context as _;
use tap::prelude::*;

use crate::util::*;
use crate::data::*;
use crate::plugin::*;
use crate::worker::*;

pub struct TaskEngine {
    plugins: HashMap<String, Plugin>,
    tasks: HashMap<TaskId, TaskWorker>,
}

impl TaskEngine {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            tasks: HashMap::new(),
        }
    }

    pub const fn tasks_mut(&mut self) -> &mut HashMap<TaskId, TaskWorker> {
        &mut self.tasks
    }

    pub fn load_config(&mut self, config_path: &Path) -> anyhow::Result<()> {
        if self.tasks.values().any(TaskWorker::is_running) {
            anyhow::bail!("cannot load config while tasks are running");
        }

        let config =
            std::fs::read_to_string(config_path)
                .pipe(none_if_not_found)
                .context("fs::read_to_string failed")?
                .map(|toml| toml::from_str::<Config>(&toml))
                .transpose()
                .context("toml::from_str failed")?
                .unwrap_or_default();
        self.tasks =
            config.tasks
                .into_iter()
                .map(|task| (task.id, TaskWorker::new(task)))
                .collect();
        Ok(())
    }

    pub fn save_config(&self, config_path: &Path) -> anyhow::Result<()> {
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
            .pipe(|toml| fs::write(config_path, &toml))
            .context("fs::write failed")?;
        Ok(())
    }

    pub fn scan_plugins(&mut self, plugins_dir: &Path) -> anyhow::Result<()> {
        self.plugins =
            scan_plugins(plugins_dir)?
                .into_iter()
                .map(|plugin| (plugin.id.clone(), plugin))
                .collect();
        Ok(())
    }
}
