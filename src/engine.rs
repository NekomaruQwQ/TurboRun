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
    pub fn new(config_path: &Path, plugin_dir: &Path) -> Self {
        let mut engine = Self {
            plugins: HashMap::new(),
            tasks: HashMap::new(),
            config_path: config_path.to_owned(),
            plugin_dir: plugin_dir.to_owned(),
        };


        // Failure to load the config is a fatal error and continuing may cause data
        // loss, so we panic instead of just logging the error.
        if let Err(err) = engine.load_config() {
            panic!("failed to load config at {}: {err:?}", config_path.display());
        }

        // Failure to scan plugins is not a fatal error: tasks that depend on missing
        // plugins will simply be invalid and won't run, but the user can still edit
        // the config and fix the problem. So we just log the error and continue.
        if let Err(err) = engine.scan_plugins() {
            log::error!("failed to scan plugins in {}: {err:?}", plugin_dir.display());
        }

        engine
    }
}

#[allow(
    dead_code,
    clippy::allow_attributes,
    reason = "accessor methods may be needed in the future")]
impl TaskEngine {
    pub const fn config_path(&self) -> &PathBuf {
        &self.config_path
    }

    pub const fn plugin_dir(&self) -> &PathBuf {
        &self.plugin_dir
    }

    pub fn tasks_sorted(&self) -> impl Iterator<Item = &TaskWorker> {
        self.tasks
            .values()
            .sorted_by_key(|worker| &worker.task().name)
    }

    pub fn plugins_sorted(&self) -> impl Iterator<Item = &Plugin> {
        self.plugins
            .values()
            .sorted_by_key(|plugin| &plugin.name)
    }

    pub fn task_status(&self, task_id: TaskId) -> TaskStatus {
        self.tasks
            .get(&task_id)
            .expect("invalid task_id")
            .status(&self.plugins)
    }

    pub fn task_is_valid(&self, task_id: TaskId) -> bool {
        self.tasks
            .get(&task_id)
            .expect("invalid task_id")
            .is_valid(&self.plugins)
    }

    pub fn create_task(&self) -> Task {
        loop {
            let task = Task::empty();
            if !self.tasks.keys().contains(&task.id) {
                break task;
            }
        }
    }

    pub fn insert_task(&mut self, task: Task) {
        self.tasks.insert(task.id, TaskWorker::new(task));
    }

    pub fn update_task(&mut self, task: Task) {
        self.tasks
            .get_mut(&task.id)
            .expect("task must already exist to be updated")
            .set_task(task);
    }

    pub fn update_or_insert_task(&mut self, task: Task) {
        if let Some(worker) = self.tasks.get_mut(&task.id) {
            worker.set_task(task);
        } else {
            self.insert_task(task);
        }
    }

    pub fn remove_task(&mut self, task_id: TaskId) {
        self.tasks.remove(&task_id);
    }
}

impl TaskEngine {
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

    /// Returns the worker for a task by ID.
    pub fn task(&self, task_id: TaskId) -> Option<&TaskWorker> {
        self.tasks.get(&task_id)
    }

    /// Polls all workers: drains output channels and collects exit status.
    /// Call once per frame.
    pub fn update(&mut self) {
        #[expect(clippy::iter_over_hash_type, reason = "update order does not matter")]
        for worker in self.tasks.values_mut() {
            worker.update();
        }
    }

    /// Starts the given task.
    ///
    /// Panics if `task_id` is not found — callers must pass a valid ID.
    pub fn run_task(&mut self, task_id: TaskId) -> anyhow::Result<()> {
        self.tasks
            .get_mut(&task_id)
            .expect("task_id must be valid")
            .run(&self.plugins)
    }

    /// Stops the given task if it is running.
    pub fn stop_task(&mut self, task_id: TaskId) {
        if let Some(worker) = self.tasks.get_mut(&task_id) {
            worker.stop();
        }
    }
}
