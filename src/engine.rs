use std::collections::*;
use std::fs;
use std::path::Path;

use crate::prelude::*;
use crate::util::*;
use crate::data::*;
use crate::plugin::*;
use crate::worker::*;

#[derive(Default)]
pub struct TaskEngine {
    tasks: HashMap<TaskId, TaskWorker>,
    plugin_packs: PluginPackMap,
    plugins: PluginMap,
}

impl TaskEngine {
    pub const fn plugins(&self) -> &PluginMap {
        &self.plugins
    }

    pub const fn plugin_packs(&self) -> &PluginPackMap {
        &self.plugin_packs
    }

    pub fn tasks_sorted(&self) -> impl ExactSizeIterator<Item = &TaskWorker> {
        self.tasks
            .values()
            .sorted_by_key(|worker| &worker.task().name)
    }

    pub fn task_status(&self, task_id: TaskId) -> TaskStatus {
        self.tasks
            .get(&task_id)
            .expect("invalid task_id")
            .status(&self.plugins)
    }

    pub fn empty_task(&self) -> Task {
        Task {
            id: TaskId::random_except(|id| self.tasks.contains_key(id)),
            name: String::from("New Task"),
            command: String::new(),
            plugins: Vec::new(),
        }
    }

    pub fn example_task(&self) -> Task {
        Task {
            id: TaskId::random_except(|id| self.tasks.contains_key(id)),
            name: String::from("Example Task"),
            command: String::from("print \"Hello, TurboRun!\""),
            plugins: vec![
                PluginInstance {
                    pack: SmolStr::new_static("base"),
                    name: SmolStr::new_static("time"),
                    enabled: true,
                    args: [("unit".into(), "s".into())].into(),
                    flags: [].into(),
                }
            ],
        }
    }

    pub fn insert_task(&mut self, task: Task) {
        self.tasks.insert(task.id, TaskWorker::new(task));
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
                .unwrap_or_else(|| Config {
                    tasks: vec![self.example_task()],
                });
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

    pub fn load_plugin_packs<'a, I>(&mut self, paths: I)
    where
        I: IntoIterator<Item = &'a Path> {
        self.plugin_packs =
            paths
                .into_iter()
                .filter_map(|path| {
                    load_plugin_pack_from_file(path)
                        .tap_err(|err| log::error!("failed to load plugin pack \"{}\": {err:?}", path.display()))
                        .ok()
                })
                .map(|pack| (pack.name.clone(), pack))
                .collect();
        self.plugins =
            self.plugin_packs
                .values()
                .flat_map(|plugin_pack| {
                    plugin_pack
                        .plugins
                        .iter()
                        .map(|plugin| ((
                            plugin_pack.name.clone(),
                            plugin.name.clone()),
                            plugin.clone()))
                })
                .collect();
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
            .run(&self.plugin_packs, &self.plugins)
    }

    /// Stops the given task if it is running.
    pub fn stop_task(&mut self, task_id: TaskId) {
        if let Some(worker) = self.tasks.get_mut(&task_id) {
            worker.stop();
        }
    }
}
