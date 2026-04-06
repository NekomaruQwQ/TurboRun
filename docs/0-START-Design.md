# TurboRun Design Document

## What is TurboRun

TurboRun is a desktop control center for managing CLI tasks on Windows. It
composes and supervises [Nushell](https://www.nushell.sh/) scripts through a
plugin-based template system, displays live output, and lets you monitor
everything from a single window.

## Why TurboRun Exists

Developer workstations accumulate always-running background jobs: dev servers,
file watchers, health checks, database proxies. The usual solutions — terminal
tabs, tmux sessions, shell scripts — all lack a single place to see what's
running, what failed, and why.

Windows Terminal profiles offer launch convenience but not lifecycle management.
They don't restart a crashed service, surface failures at a glance, or run
periodic checks. Every tab demands equal visual weight, there's no status
summary, and an accidental close can silently kill a critical process.

Existing process managers (pm2, Supervisor, Overmind) are CLI/TUI-only. The GUI
gap for local developer process management is real — no existing tool provides a
native desktop window where you glance and know what's healthy.

TurboRun fills this gap while also keeping task management logic out of your
shell config, providing clean separation of concerns.

## Design Philosophy

TurboRun follows three principles:

1. **The engine is dumb.** TurboRun manages process lifecycle — spawn, pipe,
   track, display. It has no concept of retry, health checks, timers, or
   environment setup. All behavioral richness comes from Nushell.

2. **Nu is the extension runtime.** The relationship between TurboRun and Nushell
   is like Neovim and Lua, or Emacs and Elisp. Nu is where expressiveness lives.
   TurboRun chose to compose with an existing powerful runtime rather than
   reimplement features poorly.

3. **Compose, don't configure.** Instead of accumulating optional feature flags
   on a task struct, TurboRun uses a plugin system where each "feature" is a
   plain `.nu` template that wraps the user's command. Adding capabilities means
   adding `.nu` files, not modifying Rust code.

## Architecture

### Overview

```
┌────────────────────────────────────┐
│          Main thread               │
│  ┌───────────┐  ┌───────────────┐  │     ┌─────────────────┐
│  │ egui loop  │  │ engine tick   │  │ ◄── │ pipe reader      │
│  │ render UI  │→ │ drain channel │  │ ◄── │ threads (per     │
│  │            │  │ check status  │  │ ◄── │ child process)   │
│  └───────────┘  └───────────────┘  │     └─────────────────┘
│         same struct, no sync       │
└────────────────────────────────────┘
```

The engine and UI share the main thread. The engine tick runs during each egui
update cycle (~10ms), performing only non-blocking work: draining pipe channels
and checking process status. Pipe reader threads are the only concurrency
primitive — one per active child process, pushing lines through a channel.

Engine state *is* UI state. One struct, one owner, no synchronization, no
command channels, no `Arc<Mutex<>>`, no temporal consistency bugs.

### Why egui, not web

TurboRun's UI shows a list of tasks with status indicators, live-streaming text
output, and start/stop controls. This is a real-time monitoring dashboard, not a
content-rich app with complex layouts.

With egui, the task engine and UI live in the same process, reading from the same
memory. Streaming process output to the screen is reading from a `Vec<String>`
and calling `ui.label()`. No serialization, no WebSocket, no state
synchronization across two runtimes.

A web frontend would impose a structural tax — WebSocket/SSE channels per task,
a message protocol, reconnection handling, debugging across two languages — that
is inherent to the client-server architecture, not solvable with better code.

The "looks cool" concern doesn't justify this cost. A process manager needs
clear status colors, readable monospace output, and clean spacing — not complex
layouts. egui with thoughtful theming handles this well.

### Why the engine lives on the main thread

The engine tick is microseconds of work: drain a channel, check if processes are
alive. Pipe readers are already on their own threads. There is no blocking work
to offload.

Keeping the engine on the main thread eliminates an entire class of problems:
no command channels between threads, no "UI says stop but engine hasn't
processed the message yet" bugs, no synchronization primitives. Start a task —
call a method. Render output — iterate the buffer directly.

### Why std::thread, not tokio

Tokio was natural for the web MVP because axum required it. Without a web server,
tokio provides async pipe reading and async timers — but blocking reads on
dedicated threads are simpler and equally correct at this scale, and
`Instant::elapsed() > duration` is one line. The entire async runtime, executor,
and waker system would serve no purpose. The `Send + 'static` constraints tokio
pushes onto everything would be unnecessary friction.

The runtime is `std::thread` + `crossbeam::channel`. Nothing more.

## Task Model

### From three kinds to one

The original design had three task types: Heartbeat (periodic health check),
Process Watcher (check-and-restart external processes), and Service (owned
subprocess). These were never different execution models — they were different
policies layered on the same primitive: spawn a process, observe its lifecycle.

The unified model collapses all three into a single task type. The engine
doesn't distinguish between a health check, a process watcher, or a dev server.
It spawns `nu -c "{composed script}"`, pipes output, and tracks alive/dead.
Behavioral differences live in the nu plugins that wrap the command.

### Task struct

A task definition is minimal:

- **name** — display name
- **command** — the nu command to run (fills the `{{command}}` blank)
- **plugins** — ordered list of plugin names to compose around the command
- **plugin_params** — values for each plugin's template blanks
- **last_modified** — milliseconds since Unix epoch, for change detection
- **last_reviewed** — milliseconds since Unix epoch; user must review the
  fully composed script before execution after any change

No `auto_restart`. No `interval`. No `env_vars`. No `pre_command`. No `shell`.
No `working_directory`. No `enabled`. These are all plugin concerns or future
additions, not engine concerns. The task struct does not accumulate optional
feature flags because the plugin system makes them unnecessary.

## Plugin System

### Plugins are templates

A plugin is a `.nu` file with `{{xxx}}` blanks. Composition is string
substitution — literally `str::replace` in a loop. There is no template logic,
no conditionals, no defaults, no nested evaluation. If someone needs conditional
behavior, that is nu code *inside* the template, not template-level logic.

The `{{xxx}}` syntax was chosen because double braces have no meaning in nu
(single braces are used for blocks, records, closures, and string interpolation),
it is a universally recognized convention (Mustache, Handlebars, Jinja2, Tera,
Just), and unfilled blanks cause nu's parser to reject the script before
execution — providing free validation. The reserved placeholder `{{command}}`
is used for composing the inner command into each plugin layer.

### Example plugins

**retry.nu** — periodic retry with configurable interval:
```nu
loop {
  {{command}}
  sleep {{interval}}
}
```

**healthcheck.nu** — run command then verify health:
```nu
{{command}}
if not ({{check}}) { exit 1 }
```

**env.nu** — inject environment variables:
```nu
with-env { {{env_block}} } {
  {{command}}
}
```

**pwsh.nu** — delegate to PowerShell (must be innermost plugin):
```nu
^pwsh -C "{{command}}"
```

### Composition

Plugins compose by nesting. The engine walks the plugin list inside-out,
substituting `{{command}}` at each layer. The first plugin in the list is the
innermost wrapper around the command, and the last is the outermost.

For a task with plugins `[env, retry]` and command `my-server`:

1. Start with: `my-server`
2. Apply `env` (innermost): `with-env { PORT: "3000" } { my-server }`
3. Apply `retry` (outermost): `loop { with-env { PORT: "3000" } { my-server }; sleep 5sec }`

The outermost result is what gets passed to `nu -c`.

### Preview, not magic

TurboRun always shows the fully composed nu script before execution. A preview
panel displays the final generated code. No hidden transforms — if the user can
read it, they can debug it. This is non-negotiable.

### No in-app editor

TurboRun will never include a code editor. Plugins are authored externally
(VSCode, any editor with nu support) and TurboRun references them by path. The
app's role is strictly compose and execute — competing with VSCode on editing
would be a losing battle that distracts from the core product.

## Project Structure (User Side)

```
my-workspace/
  turborun.json        # task definitions
  plugins/
    retry.nu           # loop with interval
    env.nu             # environment wrapper
    healthcheck.nu     # probe after launch
    notify-on-fail.nu  # desktop notification on exit
```

Version-controllable. Shareable. Composable. The `.nu` files are just files —
edit them anywhere, test them in a terminal independently, commit them alongside
a project.

## Nushell Binding

TurboRun explicitly depends on Nushell. Nu is not an incidental implementation
detail — it is the extension runtime and a core part of the design identity.

### Why bind to nu

The alternative — a shell-agnostic engine with feature flags for retry, env
injection, timers, etc. — would produce a tool that does a strict subset of
pm2 with a GUI. Binding to nu means the engine stays tiny (four
responsibilities) while supporting unbounded functionality through composition.
This is the architectural story that makes TurboRun interesting.

### What about non-nu users

Even shell selection is a plugin, not an engine concern. A `pwsh.nu` plugin:
```nu
^pwsh -C "{{command}}"
```

Applied as the innermost plugin (first in the list), this wraps the raw command
in a PowerShell invocation — but nu remains the universal launcher. The engine
always calls `nu -c` on the final composed script. There is no `shell` field on
the task struct, no branching in the engine. Non-nu commands are just nu invoking
an external shell via the `^` operator.

This means TurboRun degrades gracefully for non-nu users: they get raw command
execution through a shell plugin, without template composition from the broader
plugin system. The engine never needs to know the difference.

## Process Lifecycle

### Ownership model

All spawned tasks are fully owned by TurboRun. When the app exits, all running
tasks are terminated. This ensures clean shutdown with no orphaned processes
occupying ports or file locks.

There is no "detach" flag. Applications that appear to need detaching (Discord,
Thunderbird, etc.) use a launcher pattern — the exe TurboRun spawns exits
immediately after starting the real application as a separate process. TurboRun
sees a task that ran and exited with code 0. The real app was never in
TurboRun's process tree.

If a genuine need arises for a child process to outlive TurboRun, this can be
addressed then with a concrete understanding of the requirement. The clean
ownership invariant should not be undermined preemptively.

### Windows-specific note

Windows does not kill child processes on parent exit by default. The standard
mechanism for enforcing ownership is a Job Object with
`JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`. This ensures all children (and
grandchildren in the job) terminate when TurboRun closes its handle.

### Output handling

**Pipes are the primary data path.** stdout/stderr are captured through OS pipes
into reader threads that push lines through a channel to the main thread. This
is push-based — the OS wakes the reader thread exactly when data arrives.

**Log files are an optional archive, not a data source.** If persistent history
is desired, output can be teed to a file. But the UI always reads from the
in-memory buffer, never from disk.

**No run history.** Each task has one output buffer. Task starts — buffer clears.
Task exits — buffer stays until next start. One `Vec<String>` per task. If
historical output is needed, grep the log files.

File-based polling was considered and rejected. It replaces a push-based
mechanism with a pull-based one, requiring offset tracking, partial line
handling, and rotation logic — all strictly worse than pipes for live display.

## Configuration

### Format

The config file is a save file, not a source file. TurboRun has a GUI — the
primary interaction for creating and editing tasks is the window, not a text
editor. The config should be human-readable (for debugging) but need not be
optimized for human authoring.

TOML is the default choice. The data model is simple (flat list of tasks with a
few fields each), TOML round-trips trivially with serde, it is human-readable
for debugging, and it is already used for plugin metadata (`.meta.toml`
sidecars), keeping the tooling unified on a single format.

### Don't let serialization drive the data model

If a struct shape feels wrong but looks good in the config file, the config
format is exerting undue influence. The data model should serve the engine and
UI first. Serialization is a mechanical concern.

## Naming and Branding

TurboRun lives in the "Turbo" family alongside
[TurboDoc](https://github.com/NekomaruQwQ/TurboDoc). The "Turbo" prefix
references the Rust turbofish (`::<>`) — signaling the Rust tooling identity
without stating it explicitly. The turbofish serves as a shared icon motif
across the family.

"Run" describes the app's core function: it runs and monitors tasks.

## Development Decisions

### New repo, hand-crafted code

The two MVP repos used the old three-kind task model. Switching to the unified
plugin-based architecture is a rewrite from core data structures through the
engine to the UI. The MVPs served their purpose — they validated egui over web,
revealed the three-kind model was overengineered, and proved the concept. The
knowledge lives in the developer's head, not in the code.

The new repo is hand-crafted rather than heavily generated. The codebase is
small enough to hold entirely in memory. The execution engine deals with process
supervision, signal handling, and pipe management where every line has subtle
behavioral implications. Full ownership of the code is more valuable than
generation speed at this scale.

### Technology

- **Language:** Rust (2024 edition)
- **UI:** egui / eframe
- **Concurrency:** `std::thread` + `crossbeam::channel`
- **Serialization:** serde + serde_json
- **Process management:** `std::process::Command`
- **Windows integration:** `windows` crate (Job Objects)
- **Extension runtime:** Nushell
