use std::fs;
use std::path::Path;

use crate::prelude::*;
use crate::util::*;
use crate::data::*;

impl super::TaskEngine {
    pub fn load_config(&mut self, config_path: &Path) -> anyhow::Result<()> {
        if !self.task_process.is_empty() {
            anyhow::bail!("cannot load config while tasks are running");
        }

        let config =
            std::fs::read_to_string(config_path)
                .pipe(none_if_not_found)
                .context("fs::read_to_string failed")?
                .map(|toml| toml::from_str::<Config>(&toml))
                .transpose()
                .context("toml::from_str failed")?
                .unwrap_or_else(|| Config {
                    tasks: vec![self.example_task()],
                });
        self.tasks =
            config
                .tasks
                .into_iter()
                .map(|task| (task.id, task))
                .collect();
        Ok(())
    }

    pub fn save_config(&self, config_path: &Path) -> anyhow::Result<()> {
        let config = Config {
            tasks:
                self.tasks
                    .values()
                    .cloned()
                    .collect()
        };

        toml::to_string_pretty(&config)
            .context("toml::to_string_pretty failed")?
            .pipe(|toml| fs::write(config_path, &toml))
            .context("fs::write failed")?;
        Ok(())
    }
}
