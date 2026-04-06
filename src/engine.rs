use std::collections::HashMap;
use std::fs;
use std::path::Path;

use anyhow::Context as _;
use tap::prelude::*;

use crate::core::*;
use crate::plugin::*;
use crate::worker::*;

pub struct TaskEngine {
    plugins: HashMap<String, Plugin>,
    tasks: Vec<TaskWorker>,
}

impl TaskEngine {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            tasks: Vec::new(),
        }
    }

    pub fn load_config(&mut self, config_path: &Path) -> anyhow::Result<()> {
        if self.tasks.iter().any(TaskWorker::is_running) {
            anyhow::bail!("cannot load config while tasks are running");
        }

        let config =
            std::fs::read_to_string(config_path)
                .context("fs::read_to_string failed")?
                .pipe(|toml| toml::from_str::<Config>(&toml))
                .context("toml::from_str failed")?;
        self.tasks =
            config.tasks
                .into_iter()
                .map(TaskWorker::new)
                .collect();
        Ok(())
    }

    pub fn save_config(&self, config_path: &Path) -> anyhow::Result<()> {
        let tasks =
            self.tasks
                .iter()
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
