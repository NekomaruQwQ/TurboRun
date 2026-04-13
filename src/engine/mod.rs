pub use process::TaskStatus;

mod process;
use process::*;
mod plugin;
use plugin::*;
mod config;

use std::collections::*;

use garde::Validate as _;

use crate::prelude::*;
use crate::data::*;

#[derive(Default)]
pub struct TaskEngine {
    plugin_packs: PluginPackMap,
    plugins: PluginMap,
    tasks: HashMap<TaskId, Task>,
    task_process: HashMap<TaskId, TaskProcess>,
}

/// Plugin getters.
impl TaskEngine {
    pub const fn plugin_packs(&self) -> &PluginPackMap {
        &self.plugin_packs
    }

    pub const fn plugins(&self) -> &PluginMap {
        &self.plugins
    }
}

/// Task getters and views.
impl TaskEngine {
    pub fn task(&self, task_id: TaskId) -> Option<&Task> {
        self.tasks.get(&task_id)
    }

    /// Returns an iterator over all tasks and their current status,
    /// sorted by task name, ready for display in the UI.
    pub fn task_view(&self)
     -> impl ExactSizeIterator<Item = (&Task, TaskStatus)> {
        self.tasks
            .values()
            .sorted_by_key(|&task| &task.name)
            .map(|task| (task, self.task_status(task.id)))
    }

    pub fn task_status(&self, task_id: TaskId) -> TaskStatus {
        assert!(self.tasks.contains_key(&task_id), "invalid task_id");

        if self.task_is_valid(task_id) {
            self.task_process
                .get(&task_id)
                .map_or(TaskStatus::None, TaskProcess::status)
        } else {
            TaskStatus::Invalid
        }
    }

    pub fn task_is_valid(&self, task_id: TaskId) -> bool {
        // Delegates to the `garde::Validate` impl on `Task`. The structured
        // report is discarded here because callers only need a yes/no signal;
        // the editor UI calls `Task::validate_with` directly to surface
        // per-field errors.
        self.tasks[&task_id]
            .plugins
            .iter()
            .all(|inst| inst.validate_with(&self.plugins).is_ok())
    }
}

/// Task management.
impl TaskEngine {
    pub fn update_or_insert_task(&mut self, task: Task) {
        self.tasks
            .entry(task.id)
            .and_modify(|entry| *entry = task.clone())
            .or_insert(task);
    }

    pub fn remove_task(&mut self, task_id: TaskId) {
        self.tasks.remove(&task_id);
    }
}

impl TaskEngine {
    /// Polls all workers: drains output channels and collects exit status.
    /// Call once per frame.
    pub fn update(&mut self) {
        #[expect(clippy::iter_over_hash_type, reason = "update order does not matter")]
        for (&task_id, proc) in &mut self.task_process {
            proc.update(&self.tasks[&task_id].label());
        }
    }

    /// Starts the given task.
    ///
    /// Returns an error if the task is already running or if the process
    /// failed to start. Panics if `task_id` is not found — callers must
    /// pass a valid ID.
    pub fn start_task(&mut self, task_id: TaskId) -> anyhow::Result<()> {
        use std::collections::hash_map::Entry as HashMapEntry;

        if let HashMapEntry::Occupied(entry) =
            self.task_process.entry(task_id) {
            if let &TaskProcess::Running {..} = entry.get() {
                anyhow::bail!("task is already running");
            }

            entry.remove();
        }

        let task = &self.tasks[&task_id];
        let label = task.label();
        let script = apply_plugins(task, &self.plugin_packs)?;
        log::info!("running {label} with script: >>>\n{script}\n<<<");

        self.task_process
            .insert(task_id, TaskProcess::from_script(&script)?);
        Ok(())
    }

    /// Stops the given task if it is running.
    pub fn stop_task(&mut self, task_id: TaskId) -> anyhow::Result<()> {
        self.task_process
            .get_mut(&task_id)
            .context("task is not running")?
            .kill()
    }

    /// Returns the current stdout lines.
    ///
    /// While running, returns the live buffer from the active process.
    /// When stopped, returns the captured lines from the last result.
    /// Returns `&[]` if the task has never been run.
    pub fn task_stdout(&self, task_id: TaskId) -> &[String] {
        self.task_process
            .get(&task_id)
            .map_or(&[], |proc| proc.stdout())
    }

    /// Returns the current stderr lines (same semantics as `stdout()`).
    pub fn task_stderr(&self, task_id: TaskId) -> &[String] {
        self.task_process
            .get(&task_id)
            .map_or(&[], |proc| proc.stderr())
    }
}

/// Predefined tasks.
impl TaskEngine {
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
                    args: [("unit".into(), "ms".into())].into(),
                    flags: [].into(),
                }
            ],
        }
    }
}
