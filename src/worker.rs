use std::collections::HashMap;
use std::io::prelude::*;
use std::io;
use std::process::*;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use anyhow::Context as _;

use crate::data::*;
use crate::plugin::*;

pub enum TaskStatus<'a> {
    Invalid,
    Stopped,
    Running,
    Success(&'a TaskResult),
    Failure(&'a TaskResult),
}

pub struct TaskWorker {
    /// The task to be executed by this worker.
    task: Task,
    /// The [`TaskProcess`] for the currently running instance of this task,
    /// or `None` if the task is not currently running.
    proc: Option<TaskProcess>,
    /// The result of the last run, or `None` if the task has not been run yet.
    last_result: Option<TaskResult>,
}

pub struct TaskResult {
    pub elapsed: Duration,
    pub exit_code: Option<i32>,
    pub stdout: Vec<String>,
    pub stderr: Vec<String>,
}

impl TaskWorker {
    pub const fn new(task: Task) -> Self {
        Self {
            task,
            proc: None,
            last_result: None,
        }
    }

    pub const fn task(&self) -> &Task {
        &self.task
    }

    pub const fn task_mut(&mut self) -> &mut Task {
        &mut self.task
    }

    pub const fn last_result(&self) -> Option<&TaskResult> {
        self.last_result.as_ref()
    }

    pub const fn is_running(&self) -> bool {
        self.proc.is_some()
    }

    pub fn status(&self, plugins: &HashMap<String, Plugin>) -> TaskStatus<'_> {
        if !self.is_valid(plugins) {
            TaskStatus::Invalid
        } else if self.is_running() {
            TaskStatus::Running
        } else if let Some(result) = self.last_result() {
            match result.exit_code {
                Some(0) => TaskStatus::Success(result),
                Some(_) => TaskStatus::Failure(result),
                None => TaskStatus::Stopped, // Process was killed or terminated by signal
            }
        } else {
            TaskStatus::Stopped
        }
    }

    pub fn is_valid(&self, plugins: &HashMap<String, Plugin>) -> bool {
        if self.task.command.trim().is_empty() {
            return false;
        }

        for inst in &self.task.plugins {
            if !plugins.contains_key(&inst.id) {
                return false;
            }
        }

        true
    }

    pub fn update(&mut self) {
        if let Some(result) = TaskProcess::update(&mut self.proc) {
            self.last_result = Some(result);
        }
    }

    pub fn run(&mut self, plugins: &HashMap<String, Plugin>) -> anyhow::Result<()> {
        assert!(!self.is_running(), "cannot run task while it's already running");
        assert!(self.is_valid(plugins), "cannot run invalid task");

        let script =
            apply_plugins(
                plugins,
                &self.task.plugins,
                &self.task.command)?;
        log::info!("running task {} with script: >>>\n{script}\n<<<", self.task.name);
        let child =
            Command::new("nu")
                .arg("-l")
                .arg("-c")
                .arg(script)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .context("Command::spawn failed")?;
        self.proc = Some(TaskProcess::new(&self.task.name, child));
        Ok(())
    }
}

struct TaskProcess {
    name: String,
    start_time: Instant,
    child: Child,
    stdout: Vec<String>,
    stderr: Vec<String>,
    stdout_reader: PipeReader,
    stderr_reader: PipeReader,
}

impl TaskProcess {
    fn new(name: &str, mut child: Child) -> Self {
        let stdout = child.stdout.take().expect("failed to take stdout");
        let stderr = child.stderr.take().expect("failed to take stderr");
        Self {
            name: name.to_owned(),
            start_time: Instant::now(),
            child,
            stdout: Vec::new(),
            stderr: Vec::new(),
            stdout_reader: PipeReader::new(stdout),
            stderr_reader: PipeReader::new(stderr),
        }
    }

    fn update(proc: &mut Option<Self>) -> Option<TaskResult> {
        let inner = proc.as_mut()?;

        // Drain channels every tick for live output streaming.
        inner.stdout_reader.read_into(&mut inner.stdout);
        inner.stderr_reader.read_into(&mut inner.stderr);

        match inner.child.try_wait() {
            Ok(Some(status)) => {
                let mut proc = proc.take().unwrap();

                // Join threads (guarantees all lines are sent), then drain
                // any remaining lines that arrived after the last read_into.
                proc.stdout_reader.finish(&mut proc.stdout);
                proc.stderr_reader.finish(&mut proc.stderr);

                let result = TaskResult {
                    elapsed: proc.start_time.elapsed(),
                    exit_code: status.code(),
                    stdout: proc.stdout,
                    stderr: proc.stderr,
                };

                log::info!(
                    "task \"{}\" exited with status {:?} after {} ms",
                    proc.name,
                    result.exit_code,
                    result.elapsed.as_millis());

                Some(result)
            },
            Ok(None) => None,
            Err(err) => {
                eprintln!("failed to check process status: {err}");
                None
            },
        }
    }
}

struct PipeReader {
    thread: Option<thread::JoinHandle<()>>,
    rx: mpsc::Receiver<String>,
}

impl Drop for PipeReader {
    fn drop(&mut self) {
        // The thread will automatically exit when the pipe is closed and
        // the reader reaches EOF.
        // This is a no-op if `finish` was already called.
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

impl PipeReader {
    fn new(pipe: impl Read + Send + 'static) -> Self {
        let (tx, rx) = mpsc::channel();
        let thread = thread::spawn(move || {
            let reader = io::BufReader::new(pipe);
            for line in reader.lines() {
                if let Ok(line) = line {
                    tx.send(line).ok();
                } else {
                    break;
                }
            }
        });
        Self { thread: Some(thread), rx }
    }

    fn read_into<I: Extend<String>>(&self, buf: &mut I) {
        while let Ok(line) = self.rx.try_recv() {
            buf.extend(Some(line));
        }
    }

    /// Joins the reader thread and drains all remaining lines into `buf`.
    ///
    /// This guarantees no data loss: the join ensures the thread has pushed
    /// all lines, and the drain empties the channel before it's dropped.
    fn finish(mut self, buf: &mut Vec<String>) {
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
        self.read_into(buf);
    }
}
