use std::io::prelude::*;
use std::io;
use std::process::*;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
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

pub struct TaskProcess {
    child: Child,
    start_time: Instant,
    killed: bool,
    stdout: Vec<String>,
    stderr: Vec<String>,
    stdout_reader: PipeReader,
    stderr_reader: PipeReader,
}

pub struct TaskResult {
    /// The elapsed time from process start to exit.
    pub elapsed: Duration,
    /// The exit code of the process, or [`None`] if the process was killed or
    /// terminated by a signal.
    pub exit_code: Option<i32>,
    /// The captured stdout lines from the process.
    pub stdout: Vec<String>,
    /// The captured stderr lines from the process.
    pub stderr: Vec<String>,
}

impl TaskProcess {
    pub fn new(mut child: Child) -> Self {
        let stdout = child.stdout.take().expect("failed to take stdout");
        let stderr = child.stderr.take().expect("failed to take stderr");
        Self {
            child,
            start_time: Instant::now(),
            killed: false,
            stdout: Vec::new(),
            stderr: Vec::new(),
            stdout_reader: PipeReader::new(stdout),
            stderr_reader: PipeReader::new(stderr),
        }
    }

    pub fn run_script(script: &str) -> anyhow::Result<Self> {
        Command::new("nu")
            .arg("-l")
            .arg("-c")
            .arg(script)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Command::spawn failed")
            .map(Self::new)
    }

    pub const fn stdout(&self) -> &Vec<String> {
        &self.stdout
    }

    pub const fn stderr(&self) -> &Vec<String> {
        &self.stderr
    }

    pub fn kill(&mut self) -> anyhow::Result<()> {
        self.child.kill()?;
        self.killed = true;
        Ok(())
    }

    pub fn update(proc: &mut Option<Self>, label: &str) -> Option<TaskResult> {
        let proc_mut = proc;
        let proc = proc_mut.as_mut()?;

        // Drain channels every tick for live output streaming.
        proc.stdout_reader.read_into(&mut proc.stdout);
        proc.stderr_reader.read_into(&mut proc.stderr);

        match proc.child.try_wait() {
            Ok(Some(status)) => {
                let mut proc = {
                    #[expect(clippy::unwrap_in_result, reason = "guaranteed by type invariant")] {
                        proc_mut.take().unwrap()
                    }
                };

                // Join threads (guarantees all lines are sent), then drain
                // any remaining lines that arrived after the last read_into.
                proc.stdout_reader.finish(&mut proc.stdout);
                proc.stderr_reader.finish(&mut proc.stderr);

                let result = TaskResult {
                    elapsed: proc.start_time.elapsed(),
                    exit_code:
                        (!proc.killed)
                            .then_some(status.code())
                            .flatten(),
                    stdout: proc.stdout,
                    stderr: proc.stderr,
                };

                let elapsed_ms = result.elapsed.as_millis();
                if proc.killed {
                    log::info!("{label} killed after {elapsed_ms} ms");
                } else {
                    log::info!(
                        "{label} exited with status {:?} after {elapsed_ms} ms",
                        result
                            .exit_code
                            .map(|code| code.to_string())
                            .unwrap_or_else(|| String::from("<unknown>")));
                }

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
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
        self.read_into(buf);
    }
}
