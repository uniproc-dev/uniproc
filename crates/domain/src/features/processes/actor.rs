use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use app_contracts::features::agents::{
    AgentConnectionState, GpuEngineId, ProcessRunState, RemoteScanResult, SignatureStatus, WindowsAction,
    WindowsActionRequest, WindowsGpu, WindowsProcessStats, WindowsReport, WindowsReportMessage,
};
use app_contracts::features::processes::{
    Deselect, GpuEngineLabel, HostedService, MachineSummary, ProcessCategory, ProcessColumn, ProcessCommand,
    ProcessDetails, ProcessRow, ProcessStatus, ProcessesMsg, ProcessesState, RunImageCommand, RunProcessCommand,
    RunWindowCommand, Select, RunNewTask, SelectLinux, Sort, Terminate, WslEnvironment,
};
use app_contracts::features::window::PressedAway;
use guinea::prelude::*;
use tracing::instrument;
use uuid::Uuid;

use super::rates::IoRates;
use super::shell::ShellRequest;
use super::windows_scan::AppWindows;
use super::wsl_rows::environments_from_scan;

#[derive(Debug)]
pub struct ProcessesActor {
    ui_port: Push<ProcessesState>,
    rows: Rc<[ProcessRow]>,
    wsl: Rc<[WslEnvironment]>,
    machine_summary: MachineSummary,
    sort_column: ProcessColumn,
    descending: bool,
    selected: Option<u32>,
    selected_linux: Option<u32>,
    agent_state: AgentConnectionState,
    io_rates: IoRates,
    linux_rates: IoRates,
    windows: fn() -> AppWindows,
    shell: fn(ShellRequest),
    stats: std::cell::RefCell<crate::push_stats::PushStats<(Rc<[ProcessRow]>, MachineSummary, AgentConnectionState)>>,
}

impl ProcessesActor {
    pub fn new(ui_port: Push<ProcessesState>, windows: fn() -> AppWindows, shell: fn(ShellRequest)) -> Self {
        Self {
            stats: std::cell::RefCell::new(crate::push_stats::PushStats::new("processes")),
            ui_port,
            windows,
            shell,
            rows: Rc::from(Vec::new()),
            wsl: Rc::from(Vec::new()),
            machine_summary: MachineSummary::default(),
            agent_state: AgentConnectionState::Disconnected,
            io_rates: IoRates::default(),
            linux_rates: IoRates::default(),
            sort_column: ProcessColumn::Cpu,
            descending: true,
            selected: None,
            selected_linux: None,
        }
    }

    fn selected_row(&self) -> Option<&ProcessRow> {
        let pid = self.selected?;
        self.rows.iter().find(|row| row.pid == pid)
    }

    fn select_windows(&mut self, pid: Option<u32>) {
        self.selected = pid;
        self.ui_port.send(ProcessesMsg::SetSelected(pid));
        if self.selected_linux.take().is_some() {
            self.ui_port.send(ProcessesMsg::SetSelectedLinux(None));
        }
    }

    fn clear_selection(&mut self) {
        self.select_windows(None);
    }

    fn publish_wsl(&mut self, environments: Vec<WslEnvironment>) {
        if *self.wsl == *environments {
            return;
        }
        self.wsl = Rc::from(environments);
        self.ui_port.send(ProcessesMsg::SetWsl(self.wsl.clone()));
    }

    #[instrument(skip_all, level = "debug", fields(rows = self.rows.len()))]
    fn publish_rows(&self) {
        self.stats.borrow_mut().note((
            self.rows.clone(),
            self.machine_summary.clone(),
            self.agent_state,
        ));
        self.ui_port.send(ProcessesMsg::SetRows {
            rows: self.rows.clone(),
            machine: self.machine_summary.clone(),
            agent_state: self.agent_state,
        });
    }
}

const CONSOLE_HOSTS: [&str; 2] = ["conhost.exe", "OpenConsole.exe"];
const SERVICE_HOST: &str = "svchost.exe";

fn shown_name(p: &WindowsProcessStats) -> Arc<str> {
    if p.display_name.is_empty() {
        p.name.clone()
    } else {
        p.display_name.clone()
    }
}

fn is_console_host(p: &WindowsProcessStats) -> bool {
    CONSOLE_HOSTS.iter().any(|host| p.name.eq_ignore_ascii_case(host))
}

fn console_owner<'a>(
    host: &WindowsProcessStats,
    clients: &HashMap<u32, Vec<&'a WindowsProcessStats>>,
    by_pid: &HashMap<u32, &'a WindowsProcessStats>,
) -> Option<&'a WindowsProcessStats> {
    let Some(clients) = clients.get(&host.pid) else {
        return by_pid.get(&host.parent_pid).copied();
    };
    let is_client = |pid: u32| clients.iter().any(|c| c.pid == pid);
    clients
        .iter()
        .find(|c| c.pid == host.parent_pid)
        .or_else(|| clients.iter().find(|c| !is_client(c.parent_pid)))
        .or_else(|| clients.first())
        .copied()
}

fn is_service_host(p: &WindowsProcessStats) -> bool {
    p.name.eq_ignore_ascii_case(SERVICE_HOST)
}

#[instrument(skip_all, level = "debug", fields(processes = report.processes.len(), services = report.services.len()))]
pub fn rows_from_report(report: &WindowsReport, windows: &AppWindows) -> Vec<ProcessRow> {
    let has_console_hosts = report.processes.iter().any(is_console_host);
    let by_pid: HashMap<u32, &WindowsProcessStats> = if has_console_hosts {
        report.processes.iter().map(|p| (p.pid, p)).collect()
    } else {
        HashMap::new()
    };
    let mut console_clients: HashMap<u32, Vec<&WindowsProcessStats>> = HashMap::new();
    if has_console_hosts {
        for p in report.processes.iter().filter(|p| p.console_host_pid != 0) {
            console_clients.entry(p.console_host_pid).or_default().push(p);
        }
    }
    let mut known_signatures: HashMap<&str, SignatureStatus> = HashMap::new();
    for p in &report.processes {
        if p.signature != SignatureStatus::Unknown && !p.image_path.is_empty() {
            known_signatures.insert(&p.image_path, p.signature);
        }
    }
    let signature_of = |p: &WindowsProcessStats| match p.signature {
        SignatureStatus::Unknown => known_signatures
            .get(&*p.image_path)
            .copied()
            .unwrap_or(SignatureStatus::Unknown),
        known => known,
    };
    let mut hosted: HashMap<u32, Vec<HostedService>> = HashMap::new();
    for service in report.services.iter().filter(|s| s.pid != 0) {
        hosted.entry(service.pid).or_default().push(HostedService {
            name: service.name.clone(),
            display_name: service.display_name.clone(),
        });
    }
    let services: HashMap<u32, Arc<[HostedService]>> = hosted
        .into_iter()
        .map(|(pid, mut list)| {
            list.sort_by(|a, b| a.display_name.cmp(&b.display_name));
            (pid, Arc::from(list))
        })
        .collect();
    let owner_of = |p: &WindowsProcessStats| {
        if is_console_host(p) {
            return console_owner(p, &console_clients, &by_pid).map(shown_name);
        }
        if is_service_host(p) {
            return services.get(&p.pid).and_then(|list| match list.len() {
                0 => None,
                1 => Some(list[0].display_name.clone()),
                more => Some(Arc::from(format!("{} +{}", list[0].display_name, more - 1))),
            });
        }
        None
    };

    report
        .processes
        .iter()
        .map(|p| ProcessRow {
            pid: p.pid,
            cpu_percent: p.cpu_percent,
            memory_bytes: p.memory_bytes(),
            disk_bytes: p.disk_read_bytes + p.disk_write_bytes,
            net_bytes: p.net_rx_bytes + p.net_tx_bytes,
            gpu_percent: p.gpu_percent,
            gpu_memory_bytes: p.gpu_dedicated_bytes,

            exe_path: if p.image_path.is_empty() {
                p.first_arg.clone()
            } else {
                p.image_path.clone()
            },
            category: ProcessCategory::classify(
                windows.contains(p.pid),
                p.is_kernel_process,
                p.is_service,
                signature_of(p) == SignatureStatus::Microsoft,
            ),

            display_name: shown_name(p),
            name: p.name.clone(),
            package_full_name: p.package_full_name.clone(),
            owner: owner_of(p),
            owner_pid: is_console_host(p)
                .then(|| console_owner(p, &console_clients, &by_pid))
                .flatten()
                .map(|owner| owner.pid),
            services: services.get(&p.pid).cloned(),
            windows: Some(windows.of(p.pid))
                .filter(|windows| !windows.is_empty())
                .map(Arc::from),
            details: Arc::new(details(p, &report.machine.gpus)),
        })
        .collect()
}

fn status(state: &ProcessRunState) -> ProcessStatus {
    if state.suspended == Some(true) {
        ProcessStatus::Suspended
    } else if state.efficiency_mode == Some(true) {
        ProcessStatus::Efficiency
    } else {
        ProcessStatus::Running
    }
}

fn gpu_engine(engine: GpuEngineId, gpus: &[WindowsGpu]) -> Option<GpuEngineLabel> {
    let (adapter, gpu) = gpus.iter().enumerate().find(|(_, gpu)| gpu.luid == engine.adapter_luid)?;
    let name = gpu.engines.iter().find(|known| known.ordinal == engine.ordinal)?.name.clone();
    Some(GpuEngineLabel {
        adapter: adapter as u32,
        engine: name,
    })
}

fn details(p: &WindowsProcessStats, gpus: &[WindowsGpu]) -> ProcessDetails {
    ProcessDetails {
        status: status(&p.state),
        publisher: p.publisher.clone(),
        user: p.user.clone(),
        command_line: p.command_line.clone(),
        gpu_engine: p
            .gpu_engine
            .filter(|_| p.gpu_percent > 0.0)
            .and_then(|engine| gpu_engine(engine, gpus)),
        architecture: p.architecture,
        elevated: p.elevated,
        isolation: p.isolation,
    }
}

actor! {
    ProcessesActor {
        handlers { Sort, Select, SelectLinux, Deselect, Terminate, RunNewTask, RunProcessCommand, RunImageCommand, RunWindowCommand, WindowsReportMessage, RemoteScanResult, PressedAway }
    }
}

#[handler]
fn on_linux_scan(this: &mut ProcessesActor, msg: RemoteScanResult) {
    let environments = match msg {
        RemoteScanResult::Scan(scan) => {
            environments_from_scan(&scan, &mut this.linux_rates, tokio::time::Instant::now())
        }
        RemoteScanResult::Unavailable(_) => {
            this.linux_rates = IoRates::default();
            Vec::new()
        }
    };
    this.publish_wsl(environments);
}

#[handler]
fn on_windows_report(this: &mut ProcessesActor, msg: WindowsReportMessage) {
    let report = match msg {
        WindowsReportMessage::Report(report) => report,
        WindowsReportMessage::Unavailable(state) => {
            this.agent_state = state;
            this.publish_rows();
            return;
        }
    };
    this.agent_state = AgentConnectionState::Connected;
    let machine = &report.machine;
    this.machine_summary = MachineSummary {
        cpu_percent: machine.cpu_percent,
        cpu_current_mhz: machine.cpu_current_mhz,
        cpu_max_mhz: machine.cpu_max_mhz,
        memory_used_bytes: machine.used_physical_bytes(),
        memory_total_bytes: machine.total_physical_bytes,
        gpu_percent: machine.gpu_percent(),
        gpu_memory_used_bytes: machine.gpu_dedicated_used_bytes(),
        ..MachineSummary::default()
    };

    let windows = (this.windows)();
    let mut rows = rows_from_report(&report, &windows);
    this.io_rates.apply(&mut rows, tokio::time::Instant::now());
    this.rows = Rc::from(rows);
    this.publish_rows();
}

#[handler]
fn sort(this: &mut ProcessesActor, msg: Sort) {
    if this.sort_column == msg.0 {
        this.descending = !this.descending;
    } else {
        this.descending = !msg.0.sorts_ascending_first();
        this.sort_column = msg.0;
    }
    this.ui_port.send(ProcessesMsg::SetSort {
        column: this.sort_column,
        descending: this.descending,
    });
}

#[handler]
fn select(this: &mut ProcessesActor, Select(pid): Select) {
    this.select_windows(Some(pid));
}

#[handler]
fn select_linux(this: &mut ProcessesActor, SelectLinux(key): SelectLinux) {
    if this.selected.take().is_some() {
        this.ui_port.send(ProcessesMsg::SetSelected(None));
    }
    this.selected_linux = Some(key);
    this.ui_port.send(ProcessesMsg::SetSelectedLinux(this.selected_linux));
}

#[handler]
fn deselect(this: &mut ProcessesActor, _msg: Deselect) {
    this.clear_selection();
}

#[handler]
fn on_pressed_away(this: &mut ProcessesActor, _msg: PressedAway) {
    this.clear_selection();
}

#[handler]
fn run_new_task(this: &mut ProcessesActor, _msg: RunNewTask) {
    (this.shell)(ShellRequest::RunNewTask);
}

#[handler]
fn terminate(this: &mut ProcessesActor, _msg: Terminate) {
    let Some(row) = this.selected_row() else {
        return;
    };
    if !row.category.takes_actions() {
        tracing::debug!(pid = row.pid, "a kernel process is not ended");
        return;
    }
    GlobalEventBus::publish(WindowsActionRequest::new(
        Uuid::new_v4(),
        WindowsAction::Kill { pid: row.pid },
    ));
}

#[handler]
fn run_process_command(this: &mut ProcessesActor, RunProcessCommand(command): RunProcessCommand) {
    let Some(row) = this.selected_row() else {
        return;
    };
    let pid = row.pid;
    let action = match command {
        ProcessCommand::Suspend => Some(WindowsAction::Suspend { pid }),
        ProcessCommand::Resume => Some(WindowsAction::Resume { pid }),
        _ => None,
    };
    if let Some(action) = action {
        if row.category.takes_actions() {
            GlobalEventBus::publish(WindowsActionRequest::new(Uuid::new_v4(), action));
        } else {
            tracing::debug!(pid, "a kernel process is not suspended or resumed");
        }
        return;
    }
    if let Some(request) = image_request(command, &row.exe_path, &row.name) {
        (this.shell)(request);
    }
}

fn image_request(command: ProcessCommand, exe_path: &Arc<str>, name: &Arc<str>) -> Option<ShellRequest> {
    match command {
        ProcessCommand::OpenFileLocation if !exe_path.is_empty() => Some(ShellRequest::RevealFile(exe_path.clone())),
        ProcessCommand::Properties if !exe_path.is_empty() => Some(ShellRequest::FileProperties(exe_path.clone())),
        ProcessCommand::SearchOnline => Some(ShellRequest::SearchOnline(name.clone())),
        _ => None,
    }
}

#[handler]
fn run_image_command(this: &mut ProcessesActor, msg: RunImageCommand) {
    let exe_path: Arc<str> = Arc::from(msg.exe_path);
    let name: Arc<str> = Arc::from(msg.name);
    if let Some(request) = image_request(msg.command, &exe_path, &name) {
        (this.shell)(request);
    }
}

#[handler]
fn run_window_command(this: &mut ProcessesActor, msg: RunWindowCommand) {
    (this.shell)(ShellRequest::Window {
        handle: msg.handle,
        command: msg.command,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stats(pid: u32, parent_pid: u32, name: &str, display_name: &str) -> WindowsProcessStats {
        WindowsProcessStats {
            pid,
            parent_pid,
            name: name.into(),
            display_name: display_name.into(),
            ..WindowsProcessStats::default()
        }
    }

    #[test]
    fn a_console_host_is_owned_by_the_process_that_started_it() {
        let report = WindowsReport {
            processes: vec![
                stats(10, 1, "cargo.exe", ""),
                stats(11, 10, "conhost.exe", "Console Window Host"),
                stats(12, 999, "conhost.exe", "Console Window Host"),
                stats(13, 10, "rustc.exe", ""),
            ],
            ..WindowsReport::default()
        };

        let rows = rows_from_report(&report, &AppWindows::default());
        let owner = |pid: u32| rows.iter().find(|r| r.pid == pid).and_then(|r| r.owner.as_deref());

        assert_eq!(owner(11), Some("cargo.exe"));
        assert_eq!(owner(12), None, "the parent is not in this report");
        assert_eq!(owner(13), None, "only console hosts get an owner");
        assert_eq!(owner(10), None);
    }

    #[test]
    fn a_row_carries_the_details_task_manager_shows() {
        let engine = |ordinal, name: &str| app_contracts::features::agents::WindowsGpuEngine {
            ordinal,
            name: name.into(),
            ..Default::default()
        };
        let gpus = [
            WindowsGpu {
                luid: 7,
                ..Default::default()
            },
            WindowsGpu {
                luid: 9,
                engines: Arc::from([engine(0, "3D"), engine(2, "Video Decode")]),
                ..Default::default()
            },
        ];
        let suspended = ProcessRunState {
            suspended: Some(true),
            efficiency_mode: Some(true),
            ..Default::default()
        };
        let report = WindowsReport {
            processes: vec![
                WindowsProcessStats {
                    state: suspended,
                    gpu_percent: 12.0,
                    gpu_engine: Some(GpuEngineId {
                        adapter_luid: 9,
                        ordinal: 2,
                    }),
                    publisher: "Contoso".into(),
                    ..stats(10, 1, "player.exe", "")
                },
                WindowsProcessStats {
                    gpu_engine: Some(GpuEngineId {
                        adapter_luid: 9,
                        ordinal: 0,
                    }),
                    ..stats(11, 1, "idle.exe", "")
                },
            ],
            machine: app_contracts::features::agents::WindowsMachineStats {
                gpus: Arc::from(gpus),
                ..Default::default()
            },
            ..WindowsReport::default()
        };

        let rows = rows_from_report(&report, &AppWindows::default());
        let player = &rows[0].details;
        assert_eq!(player.status, ProcessStatus::Suspended, "suspended wins over efficiency mode");
        assert_eq!(&*player.publisher, "Contoso");
        assert_eq!(
            player.gpu_engine,
            Some(GpuEngineLabel {
                adapter: 1,
                engine: "Video Decode".into(),
            })
        );
        assert_eq!(rows[1].details.gpu_engine, None, "an engine that is not busy is not named");
        assert_eq!(rows[1].details.status, ProcessStatus::Running);
    }

    fn client(pid: u32, parent_pid: u32, name: &str, console_host_pid: u32) -> WindowsProcessStats {
        WindowsProcessStats {
            console_host_pid,
            ..stats(pid, parent_pid, name, "")
        }
    }

    #[test]
    fn a_console_host_belongs_to_the_process_whose_console_it_serves() {
        let report = WindowsReport {
            processes: vec![
                stats(20, 999, "conhost.exe", "Console Window Host"),
                client(21, 999, "app.exe", 20),
                stats(30, 1, "cmd.exe", ""),
                stats(31, 30, "conhost.exe", "Console Window Host"),
                client(32, 30, "cmd.exe", 31),
                client(33, 32, "cargo.exe", 31),
                stats(40, 1, "OpenConsole.exe", ""),
                client(41, 1, "pwsh.exe", 40),
            ],
            ..WindowsReport::default()
        };

        let rows = rows_from_report(&report, &AppWindows::default());
        let owner = |pid: u32| rows.iter().find(|r| r.pid == pid).and_then(|r| r.owner.as_deref());

        assert_eq!(owner(20), Some("app.exe"), "its creator is gone, the client remains");
        let owner_pid = |pid: u32| rows.iter().find(|r| r.pid == pid).and_then(|r| r.owner_pid);
        assert_eq!(owner_pid(20), Some(21));
        assert_eq!(owner_pid(31), Some(32));
        assert_eq!(owner_pid(21), None, "only console hosts point at an owner");
        assert_eq!(owner(31), Some("cmd.exe"), "the root client, not the one it started");
        assert_eq!(owner(40), Some("pwsh.exe"), "OpenConsole is a console host too");
    }

    use app_contracts::features::agents::WindowsServiceStats;

    fn service(pid: u32, display_name: &str) -> WindowsServiceStats {
        WindowsServiceStats {
            pid,
            display_name: display_name.into(),
            ..WindowsServiceStats::default()
        }
    }

    #[test]
    fn a_service_host_is_named_after_the_services_it_runs() {
        let report = WindowsReport {
            processes: vec![
                stats(20, 1, "svchost.exe", "Host Process for Windows Services"),
                stats(21, 1, "svchost.exe", "Host Process for Windows Services"),
                stats(22, 1, "svchost.exe", "Host Process for Windows Services"),
                stats(23, 1, "lsass.exe", "Local Security Authority Process"),
            ],
            services: vec![
                service(20, "Windows Audio"),
                service(21, "DCOM Server Process Launcher"),
                service(21, "RPC Endpoint Mapper"),
                service(21, "Power"),
                service(0, "Stopped Service"),
                service(23, "Security Accounts Manager"),
            ],
            ..WindowsReport::default()
        };

        let rows = rows_from_report(&report, &AppWindows::default());
        let owner = |pid: u32| rows.iter().find(|r| r.pid == pid).and_then(|r| r.owner.as_deref());

        assert_eq!(owner(20), Some("Windows Audio"));
        assert_eq!(owner(21), Some("DCOM Server Process Launcher +2"));
        assert_eq!(owner(22), None, "no running service in this host");
        assert_eq!(owner(23), None, "only generic service hosts get a name");
    }

    fn at(pid: u32, path: &str, signature: SignatureStatus) -> WindowsProcessStats {
        WindowsProcessStats {
            pid,
            name: "conhost.exe".into(),
            image_path: path.into(),
            signature,
            ..WindowsProcessStats::default()
        }
    }

    #[test]
    fn a_process_not_yet_verified_takes_the_signature_of_its_image() {
        let conhost = r"C:\Windows\System32\conhost.exe";
        let report = WindowsReport {
            processes: vec![
                at(1, conhost, SignatureStatus::Microsoft),
                at(2, conhost, SignatureStatus::Unknown),
                at(3, r"C:\tools\fresh.exe", SignatureStatus::Unknown),
            ],
            ..WindowsReport::default()
        };

        let rows = rows_from_report(&report, &AppWindows::default());
        let category = |pid: u32| rows.iter().find(|r| r.pid == pid).unwrap().category;

        assert_eq!(category(2), ProcessCategory::BackgroundMicrosoft);
        assert_eq!(category(3), ProcessCategory::BackgroundThirdParty);
    }

}
