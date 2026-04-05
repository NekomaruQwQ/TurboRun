# TurboRun Implementation Phases

This document breaks the build into incremental phases. Each phase produces a
working, testable artifact. No phase depends on a future phase — you can stop
at any point and have a functional tool.

Refer to [DESIGN.md](./DESIGN.md) for architectural rationale behind each
decision.

---

## Phase 0: Skeleton

**Goal:** Empty egui window that compiles and runs.

- Initialize an eframe application with a blank window.
- Set up the project structure: `src/main.rs`, `src/app.rs`, `src/engine.rs`,
  `src/task.rs`.
- Confirm the egui event loop runs at expected tick rate.

**Done when:** A window opens, renders nothing, and closes cleanly.

**Why this is first:** Validates the toolchain and egui setup before any logic
exists. Catches dependency issues immediately.

---

## Phase 1: Engine Core

**Goal:** Spawn a single hardcoded `nu -c "print hello"` command, capture its
output, display it in the egui window.

- Define the `Task` struct: `name`, `command`, `enabled`. No plugins yet.
- Implement process spawning: `Command::new("nu").args(["-c", &task.command])`.
- Spawn a pipe reader thread that pushes lines through a `crossbeam::channel`.
- In the engine tick (called from egui's `update`), drain the channel into a
  `Vec<String>` output buffer on the task.
- Render the output buffer in a scrollable egui panel.
- Track process status: Running / Exited(code).

**Done when:** You click a button, "hello" appears in the window, task shows as
exited with code 0.

**Why this order:** This is the irreducible core. Every future feature builds on
"spawn nu, capture output, display it." If this works, everything else is
layering.

---

## Phase 2: Start / Stop Controls

**Goal:** Interactive lifecycle management for tasks.

- Add Start and Stop buttons per task in the UI.
- Start: spawn the process, begin piping.
- Stop: kill the child process (and its process group/tree).
- Clear the output buffer on each new start.
- Handle edge cases: pressing Start while already running, pressing Stop while
  already stopped.
- Display task state visually: status indicator (e.g. green = running,
  red = exited with error, grey = stopped).

**Done when:** You can start and stop a hardcoded task repeatedly, see output
each time, and the status indicator updates correctly.

---

## Phase 3: Configuration Persistence

**Goal:** Load tasks from a JSON config file, save changes back.

- Define the on-disk JSON schema for tasks.
- Implement `load_config` / `save_config` with serde_json.
- On startup, load tasks from a configurable path (default:
  `turborun.json` in working directory).
- Add a minimal UI for creating a new task: name + command input fields.
- Add edit and delete capabilities for existing tasks.
- Save config on every mutation (add/edit/delete).

**Done when:** You can add tasks through the UI, close TurboRun, reopen it,
and see the same tasks.

**Example `turborun.json` at this phase:**
```json
{
  "tasks": [
    {
      "name": "hello",
      "command": "print 'hello from TurboRun'",
      "enabled": true
    },
    {
      "name": "dev server",
      "command": "http server --port 3000",
      "enabled": true,
      "working_directory": "C:/Projects/my-app"
    }
  ]
}
```

---

## Phase 4: Multiple Tasks and UI Layout

**Goal:** Support multiple concurrent tasks with a usable dashboard layout.

- Implement a task list sidebar showing all tasks with their status.
- Clicking a task in the sidebar shows its output in the main panel.
- Support multiple tasks running simultaneously, each with its own pipe reader
  thread and output buffer.
- Auto-scroll output to bottom, with a toggle to pin/unpin scrolling.
- Show a summary bar: e.g. "3 running, 1 failed, 2 stopped".

**Done when:** You have 3+ tasks configured, can run them concurrently, and
switch between their output views.

---

## Phase 5: Job Object Integration

**Goal:** Clean shutdown — all child processes die when TurboRun exits.

- Create a Windows Job Object on app startup using the `windows` crate.
- Set `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` on the job.
- Assign every spawned child process to the job.
- Verify: close TurboRun → all child processes terminate.
- Verify: launcher-pattern apps (e.g. apps that spawn a separate process and
  exit) are not affected.

**Done when:** You can start a long-running task, close TurboRun, and confirm
via Task Manager that the child process is gone.

**Why now:** This is the core safety invariant (TurboRun owns what it spawns).
It should be in place before adding features that spawn more processes.

---

## Phase 6: Plugin System — Template Engine

**Goal:** Compose nu templates around task commands.

- Add `plugins` (list of plugin names) and `plugin_params` (map of blank
  values) fields to the task struct.
- Implement the template engine: for each plugin in order (outermost first),
  read the `.nu` file, substitute `{{xxx}}` blanks with values from
  plugin_params, substitute `{{inner}}` with the result of the previous layer.
  Innermost layer's `{{inner}}` is the task's command.
- The engine passes the fully composed string to `nu -c`.
- Error handling: missing plugin file, unfilled blanks, missing `{{inner}}` in
  a plugin. All errors surfaced in the UI before execution.
- Configure the plugins directory path (default: `plugins/` relative to
  config file).

**Done when:** A task with `plugins: ["retry"]` and `plugin_params: { interval:
"5sec" }` produces a composed script that loops the command with a 5-second
sleep, and it runs correctly.

**Example `turborun.json` at this phase:**
```json
{
  "tasks": [
    {
      "name": "dev server",
      "command": "http server --port 3000",
      "enabled": true,
      "plugins": ["retry"],
      "plugin_params": {
        "interval": "5sec"
      }
    }
  ]
}
```

---

## Phase 7: Plugin Preview

**Goal:** Show the fully composed nu script before execution.

- Add a preview panel/toggle per task that displays the generated script.
- The preview updates live as the user changes command text, plugins, or
  params.
- Syntax-highlighted if feasible (monospace + basic keyword coloring is
  sufficient; no need for a full nu parser).
- This is a non-negotiable design requirement: no hidden transforms.

**Done when:** You can see exactly what `nu -c` will receive for any task,
including all plugin wrapping.

---

## Phase 8: Starter Plugins

**Goal:** Ship a set of useful default plugins to demonstrate the system.

Create the following `.nu` templates:

- **retry.nu** — loop with configurable interval:
  ```nu
  loop {
    {{inner}}
    sleep {{interval}}
  }
  ```
- **retry-on-failure.nu** — restart only on non-zero exit:
  ```nu
  loop {
    try { {{inner}}; break } catch { sleep {{interval}} }
  }
  ```
- **env.nu** — environment variable injection:
  ```nu
  with-env { {{env_block}} } {
    {{inner}}
  }
  ```
- **pwsh.nu** — delegate to PowerShell (innermost plugin):
  ```nu
  ^pwsh -C "{{inner}}"
  ```
- **cmd.nu** — delegate to cmd.exe (innermost plugin):
  ```nu
  ^cmd /C "{{inner}}"
  ```

**Done when:** Each starter plugin works correctly in composition. A task using
`[retry-on-failure, env]` with appropriate params runs as expected.

---

## Phase 9: Task Auto-Start

**Goal:** Optionally start tasks automatically when TurboRun launches.

- Add an `auto_start` boolean field to the task struct.
- On app startup, after loading config, start all tasks with
  `auto_start: true`.
- Respect the `enabled` field: disabled tasks are never auto-started.
- Stagger startup slightly (e.g. 100ms between spawns) to avoid a
  thundering herd of processes.

**Done when:** You open TurboRun and your dev server, file watcher, and health
check are already running without clicking anything.

---

## Phase 10: Working Directory and Plugin Config UI

**Goal:** Round out the configuration experience.

- Add `working_directory` field to the task editor UI.
- Add a plugin picker: browse available `.nu` files in the plugins directory,
  toggle them on/off per task, reorder with drag or up/down buttons.
- Add param input fields dynamically based on which blanks each selected
  plugin declares (scan the `.nu` file for `{{xxx}}` patterns, excluding
  `{{inner}}`).
- Validate params before allowing task start.

**Done when:** You can fully configure a task — command, working directory,
plugins, and all plugin params — from the UI without editing JSON.

---

## Phase 11: Log File Tee (Optional)

**Goal:** Optionally persist task output to disk.

- Add an optional `log_file` path per task (or a global log directory setting).
- The pipe reader thread tees lines to both the channel and the log file.
- Log rotation or size cap if desired, but a simple append-only file is fine
  for v1.
- The UI never reads from log files — in-memory buffer remains the primary
  data source.

**Done when:** A task runs, output appears in the UI, and the same output is
written to a file on disk.

---

## Phase 12: UI Polish

**Goal:** Make it look and feel like a finished product.

- Custom egui theme: colors, spacing, fonts. Aim for a clean monitoring
  dashboard aesthetic.
- Monospace font for output display.
- Keyboard shortcuts: start/stop selected task, switch between tasks,
  toggle preview.
- System tray integration: minimize to tray, show notification on task
  failure.
- Window state persistence: remember size, position, selected task between
  sessions.
- Error states: clear, non-technical messages when nu is not found, plugin
  file is missing, config is malformed.

**Done when:** You'd be comfortable showing this to someone in a portfolio
review.

---

## Summary

| Phase | Milestone                  | Cumulative capability                    |
|-------|----------------------------|------------------------------------------|
| 0     | Skeleton                   | Window opens                             |
| 1     | Engine core                | Spawn nu, see output                     |
| 2     | Start/stop                 | Interactive control                      |
| 3     | Config persistence         | Tasks survive restarts                   |
| 4     | Multiple tasks + layout    | Dashboard with concurrent tasks          |
| 5     | Job Object                 | Clean shutdown guarantee                 |
| 6     | Plugin template engine     | Composable nu wrappers                   |
| 7     | Plugin preview             | Transparency, debuggability              |
| 8     | Starter plugins            | Useful out of the box                    |
| 9     | Auto-start                 | Zero-click launch                        |
| 10    | Plugin config UI           | Full GUI configuration                   |
| 11    | Log file tee               | Persistent output history                |
| 12    | UI polish                  | Portfolio-ready                           |

Phases 0–5 give you a fully functional process manager. Phases 6–8 add the
plugin system that makes TurboRun architecturally interesting. Phases 9–12 are
quality-of-life and polish. You can ship or demo at any boundary.
