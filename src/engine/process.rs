use std::io::prelude::*;
use std::io;
use std::mem;
use std::process::*;
use std::sync::mpsc;
use std::thread;
use std::time::Instant;

use crate::prelude::*;

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
        child: Child,
        start_time: Instant,
        killed: bool,
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

    pub fn from_child(mut child: Child) -> Self {
        let stdout = child.stdout.take().expect("failed to take stdout");
        let stderr = child.stderr.take().expect("failed to take stderr");
        Self::Running {
            child,
            start_time: Instant::now(),
            killed: false,
            stdout: Vec::new(),
            stderr: Vec::new(),
            stdout_reader: PipeReader::new(stdout),
            stderr_reader: PipeReader::new(stderr),
        }
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
            .map(Self::from_child)
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
            &mut Self::Running { ref mut child, ref mut killed, .. } => {
                if *killed {
                    anyhow::bail!("task has already been killed");
                }
                child.kill()?;
                *killed = true;
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
                    start_time,
                    killed,
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
                    (!killed)
                        .then_some(status.code())
                        .flatten();
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
                .expect("PipeReader::finish can be called at most once")
                .join();
        self.read_into(buf);
    }
}
