use std::io::prelude::*;
use std::io;
use std::mem;
use std::process::*;
use std::sync::mpsc;
use std::thread;
use std::time::Instant;

use crate::prelude::*;

/// Creates a job object for a spawned child process so that killing the task
/// terminates the entire descendant tree, not just the direct child.
///
/// Returns `None` (with a warning log) if job creation or assignment fails;
/// the caller falls back to direct `child.kill()` in that case.
fn create_job_object_for_child(child: &Child) -> anyhow::Result<win32job::Job> {
    use std::os::windows::io::AsRawHandle as _;
    use win32job::*;

    let job_object =
        ExtendedLimitInfo::new()
            .tap_mut(|info| { info.limit_kill_on_job_close(); })
            .pipe(|info| Job::create_with_limit_info(&info))
            .context("failed to create job object")?;
    job_object
        .assign_process(child.as_raw_handle() as isize)
        .context("failed to assign child to job object")?;
    job_object.pipe(Ok)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TaskStatus {
    Invalid,
    None,
    Stopped,
    Running,
    Success,
    Failure,
}

pub enum TaskProcess {
    Running {
        /// The active child process.
        child: Child,

        /// Per-task job object that groups the child and all its descendants.
        /// Dropping this handle with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`
        /// terminates the entire process tree.
        ///
        /// [`Option`] is used to allow taking the job for killing without dropping
        /// the entire [`TaskProcess`]. This also allows detecting if the task has
        /// already been killed from call to [`TaskProcess::kill()`].
        job_object: Option<win32job::Job>,

        start_time: Instant,
        stdout: Vec<String>,
        stderr: Vec<String>,
        stdout_reader: PipeReader,
        stderr_reader: PipeReader,
    },
    Stopped {
        /// The exit code of the process as is reported by [`ExitStatus::code()`].
        exit_code: Option<i32>,
        /// The captured stdout lines from the process.
        stdout: Vec<String>,
        /// The captured stderr lines from the process.
        stderr: Vec<String>,
    },
}

impl TaskProcess {
    const fn default() -> Self {
        Self::Stopped {
            exit_code: None,
            stdout: Vec::new(),
            stderr: Vec::new(),
        }
    }

    pub fn from_child(mut child: Child) -> anyhow::Result<Self> {
        // Create a per-task job object so killing the task terminates the
        // entire process tree, not just the direct child.
        let job_object =
            create_job_object_for_child(&child)?
                .pipe(Some);
        let stdout =
            child
                .stdout
                .take()
                .context("failed to take stdout from child")?;
        let stderr =
            child
                .stderr
                .take()
                .context("failed to take stderr from child")?;
        Ok(Self::Running {
            child,
            job_object,
            start_time: Instant::now(),
            stdout: Vec::new(),
            stderr: Vec::new(),
            stdout_reader: PipeReader::new(stdout),
            stderr_reader: PipeReader::new(stderr),
        })
    }

    pub fn from_script(script: &str) -> anyhow::Result<Self> {
        Command::new("nu")
            .arg("-l")
            .arg("-c")
            .arg(script)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Command::spawn failed")
            .and_then(Self::from_child)
    }

    pub const fn stdout(&self) -> &Vec<String> {
        match self {
            &Self::Running { ref stdout, .. } |
            &Self::Stopped { ref stdout, .. } => stdout,
        }
    }

    pub const fn stderr(&self) -> &Vec<String> {
        match self {
            &Self::Running { ref stderr, .. } |
            &Self::Stopped { ref stderr, .. } => stderr,
        }
    }

    pub const fn status(&self) -> TaskStatus {
        match self {
            &Self::Running { .. } =>
                TaskStatus::Running,
            &Self::Stopped { exit_code: Some(0), .. } =>
                TaskStatus::Success,
            &Self::Stopped { exit_code: Some(_), .. } =>
                TaskStatus::Failure,
            &Self::Stopped { exit_code: None, .. } =>
                TaskStatus::Stopped,
        }
    }

    pub fn kill(&mut self) -> anyhow::Result<()> {
        match self {
            &mut Self::Running { ref mut job_object, .. } => {
                // Dropping the job with JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE
                // terminates all processes in the tree.
                job_object
                    .take()
                    .context("task has already been killed")?
                    .pipe(drop);
            },
            &mut Self::Stopped { .. } =>
                anyhow::bail!("task is not running"),
        }
        Ok(())
    }

    pub fn update(&mut self, label: &str) {
        let &mut Self::Running {
            ref mut child,
            ref mut stdout,
            ref mut stderr,
            ref mut stdout_reader,
            ref mut stderr_reader,
            ..
        } = self else {
            return;
        };

        // Drain channels every tick for live output streaming.
        stdout_reader.read_into(stdout);
        stderr_reader.read_into(stderr);

        match child.try_wait() {
            Ok(Some(status)) => {
                let Self::Running {
                    job_object,
                    start_time,
                    mut stdout,
                    mut stderr,
                    stdout_reader,
                    stderr_reader,
                    ..
                } = mem::replace(self, Self::default()) else {
                    panic!("@unreachable guaranteed by the outer pattern match");
                };

                // Join threads (guarantees all lines are sent), then drain
                // any remaining lines that arrived after the last read_into.
                stdout_reader.finish(&mut stdout);
                stderr_reader.finish(&mut stderr);

                let exit_code =
                    job_object
                        .as_ref()
                        .and_then(|_| status.code());
                let elapsed_ms =
                    start_time
                        .elapsed()
                        .as_millis();

                if let Some(exit_code) = exit_code {
                    log::info!("{label} exited with code {exit_code} after {elapsed_ms} ms");
                } else {
                    log::info!("{label} killed after {elapsed_ms} ms");
                }

                *self = Self::Stopped {
                    exit_code,
                    stdout,
                    stderr,
                };
            },
            Ok(None) => (),
            Err(err) => log::error!("failed to check process status: {err}"),
        }
    }
}

pub struct PipeReader {
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
                    tx.send(line).unwrap_or_else(|err| {
                        log::error!("failed to send data from pipe reader thread: {err}");
                    });
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
        let _ =
            self.thread
                .take()
                .expect("@assert PipeReader::finish can be called at most once")
                .join();
        self.read_into(buf);
    }
}
