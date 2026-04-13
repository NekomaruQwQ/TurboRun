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
    task_process: HashMap<TaskId, Option<TaskProcess>>,
    task_results: HashMap<TaskId, Option<TaskResult>>,
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

        if !self.task_is_valid(task_id) {
            return TaskStatus::Invalid;
        }

        if self.task_process.get(&task_id).unwrap_or(&None).is_some() {
            return TaskStatus::Running;
        }

        match self.task_results.get(&task_id).and_then(Option::as_ref) {
             Some(&TaskResult { exit_code: Some(0), .. }) =>
                TaskStatus::Success,
             Some(&TaskResult { exit_code: Some(_), .. }) =>
                TaskStatus::Failure,
             Some(&TaskResult { exit_code: None, .. }) =>
                TaskStatus::Stopped, // Process was killed or terminated by signal
             None =>
                TaskStatus::None,
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
            let task = &self.tasks[&task_id];
            if let Some(result) = TaskProcess::update(proc, &task.label()) {
                self.task_results.insert(task_id, Some(result));
            }
        }

        self.task_process.retain(|_, proc| proc.is_some());
        self.task_results.retain(|_, result| result.is_some());
    }

    /// Runs the given task.
    ///
    /// Returns an error if the task is already running or if the process
    /// failed to start. Panics if `task_id` is not found — callers must
    /// pass a valid ID.
    pub fn run_task(&mut self, task_id: TaskId) -> anyhow::Result<()> {
        let proc = self.task_process.entry(task_id).or_default();
        if proc.is_some() {
            anyhow::bail!("task is already running");
        }

        let task = &self.tasks[&task_id];
        let label = task.label();
        let script = apply_plugins(task, &self.plugin_packs)?;
        log::info!("running {label} with script: >>>\n{script}\n<<<");

        *proc = Some(TaskProcess::run_script(&script)?);
        Ok(())
    }

    /// Stops the given task if it is running.
    pub fn stop_task(&mut self, task_id: TaskId) {
        if let Some(&mut Some(ref mut proc)) = self.task_process.get_mut(&task_id) {
            proc.kill()
                .unwrap_or_else(|err| log::error!("failed to kill task {task_id}: {err:?}"));
        }
    }

    /// Returns the current stdout lines.
    ///
    /// While running, returns the live buffer from the active process.
    /// When stopped, returns the captured lines from the last result.
    /// Returns `&[]` if the task has never been run.
    pub fn task_stdout(&self, task_id: TaskId) -> &[String] {
        if let Some(proc) =
            self.task_process
                .get(&task_id)
                .and_then(Option::as_ref) {
            proc.stdout()
        } else {
            self.task_results
                .get(&task_id)
                .unwrap_or(&None)
                .as_ref()
                .map_or(&[], |result| &result.stdout)
        }
    }

    /// Returns the current stderr lines (same semantics as `stdout()`).
    pub fn task_stderr(&self, task_id: TaskId) -> &[String] {
        if let Some(proc) =
            self.task_process
                .get(&task_id)
                .and_then(Option::as_ref) {
            proc.stderr()
        } else {
            self.task_results
                .get(&task_id)
                .and_then(Option::as_ref)
                .map_or(&[], |result| &result.stderr)
        }
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
