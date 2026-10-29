# AGENTS.md

## What this is

Uniproc is a task manager for Windows 11 and WSL: processes, services, machine metrics,
WSL distributions. The UI is WinUI 3, driven from Rust through `windows-reactor`, with
`guinea` on top for routing, features, actors and reducers.

Uniproc reads nothing heavy itself. Numbers come from two out-of-process agents over
capnp-rpc (`ogurpchik`):

- **the Windows service** (`uniproc-windows-agent`, LocalSystem, session 0) — processes,
  services, machine;
- **the Linux agent** inside a WSL distribution (`uniproc-linux-agent`, eBPF) — Linux
  processes, containers, the WSL machine.

Windows is the only target. No `cfg(windows)` gates, no fallbacks. "Linux" in the code
means the WSL agent the Windows host talks to.

Taskmgr is the reference for what to show, not for how it gets it wrong: where it is wrong
we show the right thing.

## Crates

| crate | owns |
|---|---|
| `app-contracts` | What domain and UI agree on: rows, states, reducers, messages, actions. No behaviour, no platform calls. |
| `domain` | Behaviour. One module per feature: actor, installer, settings, platform code. |
| `ui` | Views. Pages, widgets, theme, formatting. Reads state, dispatches actions. |
| `context` | Icon and metadata extraction from executables, packages and windows. |
| `desktop` | The binary: routes, layouts, page wiring, tracing, test fakes. |
| `xtask` | Dev tasks. |

`desktop` → `ui` → `app-contracts` ← `domain`. `ui` never depends on `domain`.

`desktop` is scaffolding and will be rewritten once guinea settles; keep it working, do not
polish it.

Tables use native Views through ItemsRepeater and ScrollViewer in `ui::widgets::table`.
Services columns are catalogued by `ServiceColumn::ALL` with stable ids and default
visibility and widths. The overview and settings page use that same catalogue;
`ServicesSettings` persists visibility, widths and order. `ServicesArea` owns the feature
across both routes. Service startup type, account and control capabilities come from
the Windows service, never from SCM calls made by the UI.

## A feature

| layer | file |
|---|---|
| contract | `app-contracts/src/features/<f>/{model,state,messages}.rs` |
| behaviour | `domain/src/features/<f>/{actor,install}.rs` |
| view | `ui/src/pages/<f>/` |
| wiring | `desktop/src/pages/<f>.rs` or a layout in `desktop/src/layouts/` |

- **State** is a `#[reducer]` over `<F>Msg`. Only the feature's actor sends `<F>Msg`,
  through its `Push<State>` port.
- **Actions** are `#[derive(guinea::Remote)] #[remote(action)]` structs. The view
  dispatches them; the actor handles them.
- **The installer** (`#[installs]`) builds the state with `cx.state::<S>().driven_by(..)`,
  starts timers with `cx.every(..)`. The actor declares bus subscriptions in `subscribes`
  and publications in `publishes`; its handlers use `In<Actor, Message>` and `cx.task()`.
  Long-lived work belongs to the installation scope; dropping a source handle does not
  stop it. The bus belongs to the application tree, not the process.
- **`<F>Deps`** (`ProcessesDeps`, `WslDeps`, `SystemDeps`, `AgentLinkDeps`) is what the
  installer calls into the platform with. `Default` is the real thing; tests swap fakes.
  The `Platform` plugin in `desktop/src/platform.rs` declares these values in `provides`;
  its parameters select real platform functions or test fakes. A page or layout receives
  its dependency as an injected `install` argument and passes it as feature
  parameters; the installer reads `cx.params()` from `Install<FDeps>`.
  It is not route params. `StorePlugin` automatically publishes the settings `Store`,
  which is injected explicitly into feature installers. No `provide` or service lookup.
- **Where it is installed decides how long it lives.** A page's own state goes in the page
  or in the area layout above it (`ProcessesArea`, `SystemArea`), never in `Shell`: the
  shell installs only what the shell itself shows (the sidebar). A separately mounted
  part has its own feature (`SidebarCharts` → metrics, `Connecting` →
  agent link); window-wide state (settings) is in `MainWindow`. 
  App-lifetime features use `feature!` and a `#[installs]` function taking
  `&mut FeatureBuilder`. The application function in `desktop/src/app.rs` installs
  them (`agents`, `activity`) with `app.install::<F>()`, builds plugins with explicit
  configs and returns the initial WinUI `Window`. `provides` outputs are published by
  guinea only after their installer succeeds; use returned values for startup wiring.
  App state is claimed with `app.state` and listed in the feature's `exports`.

Routes are a `Routable` enum in `desktop/src/routes.rs`. Layouts explicitly mount their
children and select routes with `cx.outlet`. `MainWindow` is the root; the
last route survives a restart, see `route_memory.rs`. It owns the window (size, backdrop,
theme, colour scheme) and holds `Shell`, which draws the title bar and navigation.
The shell mounts parts (`desktop/src/parts/`) for its footer and overlay. They redraw
on their own: a machine sample redraws `SidebarCharts`, not the shell or the page.
`ProcessesArea` is mounted with `.keep()`: leaving it
puts its scope to sleep instead of tearing it down, so coming back shows the rows at once.
Asleep, its timers and bus subscriptions are paused; `cx.on_wake` is where a feature
catches up (Processes resets its rates and asks for the service state).
Ordinary tasks can finish and deliver replies while asleep; sleep is not cancellation.

## Agents

`domain/features/agents` owns the connections: one `GenericAgentActor<B>` per backend
(`WindowsBackend`, `WslBackend`), with `ConnectionMachine` as its state machine.

- Connecting: one attempt per window (`connect_attempt_secs`, 3 s). After 5 failures the
  state is `GaveUp`, and attempts continue. A service too old to `watch` is `Outdated`, also
  retried. A connection lost before one window has passed waits out the rest of it.
- Data is pushed, not polled: once connected the actor runs the backend's streams with
  `cx.task().pour`. Backend operations receive the actor's `AsyncContext`; reports
  land on that application's bus as `WindowsReportMessage`,
  `WindowsMachineSample` and `RemoteScanResult`; features subscribe to what they need.
- Windows: `WindowsFeed` and `windows_report::Reports` turn the service's lists and columns
  into `WindowsReport`. The service runs in session 0 and cannot see user windows; those
  come from `domain/features/processes/windows_scan.rs`, in-process.
- WSL: uniproc starts the agent through `wsl.exe` (distro and path from `AgentSettings`)
  and connects over vsock. `linux_report::LinuxReports` applies `watch` updates (full,
  delta against a `baseEtag`, unchanged); anything it cannot apply asks for a resync.
  Disk is the agent's file I/O (`fileRead/WriteBytes`), not the block layer.
- When the service does not answer, the splash offers "Open monitor in process" after 5 s.
  The monitor in process is the same Windows agent reached another way: `agent_link`
  publishes `RunWindowsAgentInProcess`, and the Windows agent actor switches its source
  from the service to `AgentBackend::connect_in_process` (`Agent::local()`, the same
  `WindowsClient` and `WindowsFeed`), runs the same streams and announces the same
  transport. It stays in process for the rest of the run and answers with
  `WindowsAgentInProcess` (`Running`, `NotElevated`, `Failed`); a start that fails hands
  back to the service. A switch bumps the actor's epoch, so answers and timers left from
  the old source are dropped. `agent_link` itself holds only the splash: the offer, the
  elevation and the restart.
  `Local` needs an elevated uniproc. Without it the button carries a shield and restarts
  uniproc through `runas` with `--in-process --after <pid>`, then closes this window; the
  new copy waits in `main` for that pid to exit (the single-instance lock is the old
  copy's until then) and starts the monitor at once. A refused UAC prompt leaves things
  as they were. `AgentLinkDeps::elevation` is that platform side; tests fake it.
- Actions on processes and services are an RPC: `WindowsActionRequest(action)` answered
  with `ActionOutcome`, asked through `agents::actions::request`. Only `WindowsActions`
  answers. It hears the current transport (`WindowsTransport::Connected`, announced by
  the Windows agent actor on connect, `Lost` on loss), whichever source it came from, and
  calls `act` on it through `ActsOnWindows`. Guinea allows
  one answerer per request type and panics on a second; with none, the request fails
  at once without being published. Page tests that act fake the service with
  `h.bus().answer_fn`, keeping the returned subscription for the test's lifetime.
- Uniproc never offers to end or suspend itself or its service (`ProcessRow::is_monitor`).
- Process starts and exits are `WindowsProcessEvents` (a history on connect, then live
  batches), keyed by `ProcessInstance` (pid, sequence), from the service's
  `watch_process_events` (or the monitor in process). `activity` keeps the log for the
  whole run; batches only record, a 250 ms flush publishes the view, and the view builds
  full rows only for the newest 200.

UI text says **service**, never "agent".

## UI

```
ui/src/pages/<name>/
  mod.rs          modules and re-exports only
  page.rs         the page view
  <sub>.rs        a second page sharing this page's components (processes/settings.rs)
  marks.rs        the page's marks
  components/     private to the page
```

- Nothing in one page's `components` is reachable from another page. What two pages share
  goes to `widgets/`; what one page uses stays in its `components`.
- Views get `&L10n` and a `Palette` from the desktop page, which calls `use_tr(cx)` and
  `Palette::of(..)`; ui views have no `cx`.
- Marks (`#[derive(guinea::Mark)]`) are how tests find things. Add one when a test needs it.

### Theme

`ui/src/theme/`: `space`, `size`, `radius`, `opacity`, `palette`.

- A token is a value that must match **between** components. Arithmetic inside one
  component stays a local constant in that component.
- System brushes: `ThemeBrush` where it has one. The pinned reactor has only eight, so the
  rest of WinUI's named brushes are in `Palette`, light and dark, until the reactor exposes
  them. Never a bare colour constant: it has one value for both themes.
- Type: the factories in `widgets/text.rs` (`text`, `caption`, `body_strong`, `body_large`,
  `subtitle`). They are the only place with `.font_size(..)`.
- Bytes, rates and percents are formatted in `ui/src/format.rs`.

### Localization

`locales/en/` mirrors where a string is shown: `common.ftl`, `layouts/shell.ftl`,
`pages/<page>.ftl`, `widgets/<widget>.ftl`. `taskmgr.ftl` holds strings taken from
Taskmgr's own resources by `cargo run -p xtask -- l10n-taskmgr`.

- Ids are global; the file-name prefix is the namespace (`processes-col-name`).
- Do not share a string because two surfaces spell it the same in English.
- Sentences, counts and separators are composed in Fluent, not with `format!`.
- Never use a localized string as a key. Persisted enums have `id()`; display goes through
  a label function.

## Settings

Open settings with `StoreAccess::try_settings` and return opening errors from
installers and startup through `Result`. `Store` is a required dependency;
do not hide missing settings behind optional injection or use panicking `settings`.

Settings are `#[amethystate(prefix = ..)]` structs per feature, stored by
`guinea-plugin-store`. There are no users yet: when a key changes, drop the old one; no
migrations or compatibility shims for old local data.

## Generated, do not edit

- `app-contracts/src/icons.rs` and the l10n accessors — `app-contracts/build.rs`, from
  `icons.gui.toml` (+ `icons.vendor.gui.toml`) and `locales/`.
- Icon SVGs: fetched by guicons into `.cache/guicons/`, pinned by `icons.lock`. Both are
  committed.
- App metadata — `desktop/build.rs`, from `app.toml`.
- `<crate>/src/bindings.rs` in `context`, `domain`, `ui`, `xtask` — `cargo run -p xtask --
  bindings`, from `bindings.txt` next to it; an xtask test fails when one is out of date.
- `desktop/src/window_press/bindings.rs` — `cargo run -p xtask --
  winui-bindings <winmd dir>`, from `bindings.txt` next to it. The WinUI metadata is
  `crates/tools/reactor-metadata/winmd` of a windows-rs checkout at the commit
  `windows-reactor-pre` was published from; without the argument the command names that
  commit. That commit is not always on GitHub; the last upstream commit to change the
  directory before it has the same files.
- Anything under `target/`.

## Tests

`cargo test --workspace`. Behaviour is proved by tests, not by clicking through the app.

- UI behaviour: guinea harness tests in `desktop` (`#[guinea::test(iterations = ..)]`). They run in
  parallel; a test that touches a fake's statics takes its key: `exclusive = "agent"` for
  `test_agent`, `exclusive = "system"` for `test_system`.
  A page: `h.install::<F>(..)` + `Mounted::mount_at(h.child(), ..)`. The real tree:
  `h.app(|app| crate::app::with_fakes(app, deps))` (in-memory store, fake agent) and
  `Mounted::from_window(h, window)`; read and drive it with `state`, `dispatch`, `route`,
  `is_mounted`. No probe pages.
- Fakes: `PlatformDeps` parameters of the same `Platform` plugin as production;
  `test_support::install_with` installs it with an in-memory store and localisation.
  Keep the returned `Store` for assertions, rather than looking it up from the harness.
  `test_agent.rs` is a fake Windows agent
  (`FakeAgentFeature`, switches for up, outdated, dropping); `test_system.rs` for the
  System page.
- Time is virtual: `h.advance(..)`, then `settle()`.
- Don't test guinea, the harness or the reactor; don't keep snapshot lists that change with
  every feature.
- `app::tests::application_declarations_are_valid` checks the linked application graph
  through `guinea::app::validate_declarations`; harness installation alone does not run
  this global check. Production and tests share one provider declaration per type.

## Running

- `cargo run` (or `cargo rdesk`) starts the app; the Windows service must be installed and
  running. `cargo ragent` waits for a manually started service, then runs the app.
- `cargo run -p xtask -- agent-check [--wsl]` runs `domain/examples/agent_e2e.rs` against the
  live agents.
- Logs: plain text on stderr, and one JSON object per line in `run_desktop.log` (`ts`,
  `level`, `target`, `cause`, fields), recreated on every start. Debug builds write it in
  the working directory; release builds in `%LOCALAPPDATA%\<app name>\logs\`.
  `trace-scopes.toml` configures scopes.
- Environment (debug builds):
  - `UNIPROC_AGENT_PIPE=<name>` — another Windows service pipe name.
  - `UNIPROC_NO_DEVTOOLS` — don't launch guinea devtools.
  - `UNIPROC_SYNTHETIC_AGENT=<n>` (+ `UNIPROC_SYNTHETIC_UNIQUE`, `UNIPROC_SYNTHETIC_JITTER`) —
    a generated Windows report of `n` processes instead of the service.
  - `dhat-heap` feature + `UNIPROC_DHAT_FILE` — heap profile.

## Dependencies

- Our crates come from two organisations: `guinea-rs` (guinea, guinea-plugins, guicons,
  amethystate) and `uniproc-dev` (the agents, `uniproc-protocol`, `ogurpchik`). Each repo
  has its own owner; problems found there are reported to it, not patched here.
- No `windows` crate: each crate generates the Win32/WinRT items it calls with
  windows-bindgen 0.100 over `windows-core` 0.100. The reactor crates come from crates.io
  (`-pre`).
- CI: `.github/workflows/deps.yml` calls the shared `guinea-rs/.github` workflows
  (cargo-deny with `deny.toml`, one version of each of our crates, a weekly issue listing
  newer tags). Dependabot ignores our own crates; those are bumped by hand.

## Code style

- No comments in code. Rationale goes in the commit message or here.
- Destructure messages in a handler's signature.
- Constants that belong together live in a namespace (`impl Pace { const Report: .. }`).
