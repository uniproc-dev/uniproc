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
  subscribes the actor to the global bus, and starts timers with `cx.every(..)`. Anything
  long-lived belongs to the scope it was installed in — a dropped handle stops silently.
- **`<F>Deps`** (`ProcessesDeps`, `WslDeps`, `SystemDeps`, `AgentLinkDeps`) is what the
  installer calls into the platform with. `Default` is the real thing; tests swap fakes.
  The segment takes it with `ctx.require_or_default::<FDeps>()`. It is not route params.
- **Where it is installed decides how long it lives.** A page's own state goes in the page
  or in the area layout above it (`ProcessesArea`, `SystemArea`), never in `ShellLayout`:
  the shell installs only what the shell itself shows (sidebar, metrics, agent link,
  settings). App-lifetime features implement `AppFeature` and go in `main.rs` (`agents`).

Routes are in `desktop/src/routes.rs`. `ShellLayout` is the root and is `restorable`: the
last route survives a restart, see `route_memory.rs`. `ProcessesArea` is `keep`: leaving it
puts its scope to sleep instead of tearing it down, so coming back shows the rows at once.
Asleep, its timers and bus subscriptions are paused; `cx.on_wake` is where a feature
catches up (Processes resets its rates and asks for the service state).

## Agents

`domain/features/agents` owns the connections: one `GenericAgentActor<B>` per backend
(`WindowsBackend`, `WslBackend`), with `ConnectionMachine` as its state machine.

- Connecting: one attempt per window (`connect_attempt_secs`, 3 s). After 5 failures the
  state is `GaveUp`, and attempts continue. A service too old to `watch` is `Outdated`, also
  retried. A connection lost before one window has passed waits out the rest of it.
- Data is pushed, not polled: once connected the actor runs the backend's streams with
  `spawn_source`. Reports land on the global bus as `WindowsReportMessage`,
  `WindowsMachineSample` and `RemoteScanResult`; features subscribe to what they need.
- Windows: `WindowsFeed` and `windows_report::Reports` turn the service's lists and columns
  into `WindowsReport`. The service runs in session 0 and cannot see user windows; those
  come from `domain/features/processes/windows_scan.rs`, in-process.
- WSL: uniproc starts the agent through `wsl.exe` (distro and path from `AgentSettings`)
  and connects over vsock. `linux_report::LinuxReports` applies `watch` updates (full,
  delta against a `baseEtag`, unchanged); anything it cannot apply asks for a resync.
  Disk is the agent's file I/O (`fileRead/WriteBytes`), not the block layer.
- When the service does not answer, the splash offers "Open monitor in process" after 5 s.
  `agent_link` then runs `uniproc_windows_agent::local::Local` inside uniproc (needs an
  elevated uniproc, otherwise `NotElevated`) and publishes `WindowsAgentInProcess`: the
  service actor goes dormant and `agent_link` answers for it.
- Actions on processes and services are an RPC: `WindowsActionRequest(action)` answered
  with `ActionOutcome`, asked through `agents::actions::request`. Only `WindowsActions`
  answers. It hears which transport is current — the service (`WindowsTransport::Remote`,
  announced by the service actor on connect and on loss) or the monitor in process
  (`Local`, which wins for the rest of the run) — and calls `act` on it. Guinea allows
  one answerer per request type and panics on a second; with none, the request fails
  at once without being published. Page tests that act fake the service with
  `GlobalEventBus::answer_fn`.
- Uniproc never offers to end or suspend itself or its service (`ProcessRow::is_monitor`).

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

Settings are `#[amethystate(prefix = ..)]` structs per feature, stored by
`guinea-plugin-store`. There are no users yet: when a key changes, drop the old one; no
migrations or compatibility shims for old local data.

## Generated, do not edit

- `app-contracts/src/icons.rs` and the l10n accessors — `app-contracts/build.rs`, from
  `icons.gui.toml` (+ `icons.vendor.gui.toml`) and `locales/`.
- Icon SVGs: fetched by guicons into `.cache/guicons/`, pinned by `icons.lock`. Both are
  committed.
- App metadata — `desktop/build.rs`, from `app.toml`.
- `desktop/src/window_press/bindings.rs` — `cargo run -p xtask -- winui-bindings`, from
  `bindings.txt` next to it.
- Anything under `target/`.

## Tests

`cargo test --workspace`. Behaviour is proved by tests, not by clicking through the app.

- UI behaviour: guinea harness tests in `desktop` (`#[guinea::test(iterations = .., exclusive = "store")]`).
  A page: `h.install::<F>(..)` + `Below::mount(h, |below| Mounted::mount_at(below, ..))`
  (`test_page.rs`): a dropped `h.child()` tears its scope down under the mounted page. A layout:
  `Mounted::mount_at(&h.segment(), ..)`; its outlet is `Outlet`, navigation is read with
  `navigated()`. No probe pages.
- Fakes: `<F>Deps` through `h.provide(..)`; `test_agent.rs` is a fake Windows agent
  (`FakeAgentFeature`, switches for up, outdated, dropping); `test_system.rs` for the
  System page.
- Time is virtual: `h.advance(..)`, then `settle()`.
- Don't test guinea, the harness or the reactor; don't keep snapshot lists that change with
  every feature.

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
- `windows` stays on a git rev of microsoft/windows-rs until 0.100 is published; the
  reactor crates come from crates.io (`-pre`).
- CI: `.github/workflows/deps.yml` calls the shared `guinea-rs/.github` workflows
  (cargo-deny with `deny.toml`, one version of each of our crates, a weekly issue listing
  newer tags). Dependabot ignores our own crates; those are bumped by hand.

## Code style

- No comments in code. Rationale goes in the commit message or here.
- Destructure messages in a handler's signature.
- Constants that belong together live in a namespace (`impl Pace { const Report: .. }`).
