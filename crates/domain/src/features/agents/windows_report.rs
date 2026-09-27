use std::collections::HashMap;
use std::sync::Arc;

use app_contracts::features::agents::{
    ProcessPriority, SignatureStatus, WindowsAction, WindowsMachineStats, WindowsProcessStats,
    WindowsReport, WindowsServiceState, WindowsServiceStats,
};
use uniproc_windows_agent::api;

#[derive(Default)]
pub struct Reports {
    processes: Option<api::Tagged<Arc<[api::ProcessInfo]>>>,
    passports: HashMap<u32, WindowsProcessStats>,
    services: Option<(u64, Vec<WindowsServiceStats>)>,
}

impl Reports {
    pub fn report(&mut self, snapshot: &api::Snapshot) -> WindowsReport {
        self.keep_passports(&snapshot.processes);
        self.keep_services(&snapshot.services);

        let processes = snapshot
            .metrics
            .iter()
            .filter_map(|metrics| {
                let passport = self.passports.get(&metrics.pid)?;
                Some(with_metrics(passport.clone(), metrics))
            })
            .collect();

        WindowsReport {
            machine: machine(&snapshot.machine),
            processes,
            services: self.services.as_ref().map(|(_, services)| services.clone()).unwrap_or_default(),
        }
    }

    fn keep_passports(&mut self, processes: &api::Tagged<Arc<[api::ProcessInfo]>>) {
        if self.processes.as_ref().is_some_and(|held| held.etag == processes.etag) {
            return;
        }
        let before: HashMap<u32, &api::ProcessInfo> = self
            .processes
            .as_ref()
            .map(|held| held.value.iter().map(|info| (info.pid, info)).collect())
            .unwrap_or_default();

        let mut held = std::mem::take(&mut self.passports);
        self.passports = processes
            .value
            .iter()
            .map(|info| {
                let kept = held
                    .remove(&info.pid)
                    .filter(|_| before.get(&info.pid).is_some_and(|was| *was == info));
                (info.pid, kept.unwrap_or_else(|| passport(info)))
            })
            .collect();
        self.processes = Some(processes.clone());
    }

    fn keep_services(&mut self, services: &api::Tagged<Arc<[api::ServiceStats]>>) {
        if self.services.as_ref().is_some_and(|(etag, _)| *etag == services.etag) {
            return;
        }
        self.services = Some((services.etag, services.value.iter().map(service).collect()));
    }
}

fn passport(info: &api::ProcessInfo) -> WindowsProcessStats {
    WindowsProcessStats {
        pid: info.pid,
        parent_pid: info.parent_pid,
        session_id: info.session_id,
        name: Arc::from(info.name.as_str()),
        first_arg: Arc::from(info.cmdline.first().map_or("", String::as_str)),
        package_full_name: Arc::from(info.package_full_name.as_str()),
        package_relative_app_id: Arc::from(info.package_relative_app_id.as_str()),
        is_service: info.is_service,
        is_kernel_process: info.is_kernel_process,
        is_windows_process: info.is_windows_process,
        signature: signature(info.signature),
        image_path: Arc::from(info.image_path.as_str()),
        display_name: Arc::from(info.display_name.as_str()),
        console_host_pid: info.console_host_pid,
        ..WindowsProcessStats::default()
    }
}

fn with_metrics(passport: WindowsProcessStats, m: &api::ProcessMetrics) -> WindowsProcessStats {
    WindowsProcessStats {
        cpu_percent: m.cpu_percent,
        working_set_kb: m.working_set_kb,
        private_bytes_kb: m.private_bytes_kb,
        peak_working_set_kb: m.peak_working_set_kb,
        private_working_set_kb: m.private_working_set_kb,
        disk_read_bytes: m.disk_read_bytes,
        disk_write_bytes: m.disk_write_bytes,
        disk_read_iops: m.disk_read_iops,
        disk_write_iops: m.disk_write_iops,
        net_rx_bytes: m.net_rx_bytes,
        net_tx_bytes: m.net_tx_bytes,
        ..passport
    }
}

fn machine(m: &api::MachineStats) -> WindowsMachineStats {
    WindowsMachineStats {
        total_physical_kb: m.total_physical_kb,
        available_physical_kb: m.available_physical_kb,
        used_physical_kb: m.used_physical_kb,
        cpu_percent: m.cpu_percent,
        cpu_max_mhz: m.cpu_max_mhz,
        cpu_current_mhz: m.cpu_current_mhz,
        disk_read_bytes: m.disk_read_bytes,
        disk_write_bytes: m.disk_write_bytes,
        disk_read_iops: m.disk_read_iops,
        disk_write_iops: m.disk_write_iops,
        net_rx_bytes: m.net_rx_bytes,
        net_tx_bytes: m.net_tx_bytes,
    }
}

fn service(s: &api::ServiceStats) -> WindowsServiceStats {
    WindowsServiceStats {
        name: Arc::from(s.name.as_str()),
        display_name: Arc::from(s.display_name.as_str()),
        pid: s.pid,
        state: service_state(s.state),
        load_group: Arc::from(s.load_group.as_str()),
        description: Arc::from(s.description.as_str()),
        image_path: Arc::from(s.image_path.as_str()),
    }
}

fn signature(status: api::SignatureStatus) -> SignatureStatus {
    match status {
        api::SignatureStatus::Unknown => SignatureStatus::Unknown,
        api::SignatureStatus::Unsigned => SignatureStatus::Unsigned,
        api::SignatureStatus::Microsoft => SignatureStatus::Microsoft,
        api::SignatureStatus::ThirdParty => SignatureStatus::ThirdParty,
    }
}

fn service_state(state: api::ServiceState) -> WindowsServiceState {
    match state {
        api::ServiceState::Unknown => WindowsServiceState::Unknown,
        api::ServiceState::Stopped => WindowsServiceState::Stopped,
        api::ServiceState::StartPending => WindowsServiceState::StartPending,
        api::ServiceState::StopPending => WindowsServiceState::StopPending,
        api::ServiceState::Running => WindowsServiceState::Running,
        api::ServiceState::ContinuePending => WindowsServiceState::ContinuePending,
        api::ServiceState::PausePending => WindowsServiceState::PausePending,
        api::ServiceState::Paused => WindowsServiceState::Paused,
    }
}

fn priority(priority: ProcessPriority) -> api::ProcessPriority {
    match priority {
        ProcessPriority::Idle => api::ProcessPriority::Idle,
        ProcessPriority::BelowNormal => api::ProcessPriority::BelowNormal,
        ProcessPriority::Normal => api::ProcessPriority::Normal,
        ProcessPriority::AboveNormal => api::ProcessPriority::AboveNormal,
        ProcessPriority::High => api::ProcessPriority::High,
        ProcessPriority::Realtime => api::ProcessPriority::Realtime,
    }
}

pub fn command(action: WindowsAction) -> api::Command {
    match action {
        WindowsAction::Kill { pid } => api::Command::Kill { pid },
        WindowsAction::Suspend { pid } => api::Command::Suspend { pid },
        WindowsAction::Resume { pid } => api::Command::Resume { pid },
        WindowsAction::SetPriority { pid, priority: p } => api::Command::SetPriority { pid, priority: priority(p) },
        WindowsAction::SetAffinity { pid, mask } => api::Command::SetAffinity { pid, mask },
        WindowsAction::ServiceStart { name } => api::Command::ServiceStart { name },
        WindowsAction::ServiceStop { name } => api::Command::ServiceStop { name },
        WindowsAction::ServicePause { name } => api::Command::ServicePause { name },
        WindowsAction::ServiceResume { name } => api::Command::ServiceResume { name },
        WindowsAction::ServiceRestart { name } => api::Command::ServiceRestart { name },
    }
}

pub fn code(result: anyhow::Result<api::CommandResult>) -> u32 {
    match result {
        Ok(Ok(())) => 0,
        Ok(Err(code)) => code,
        Err(_) => u32::MAX,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn info(pid: u32, name: &str, cmdline: &[&str]) -> api::ProcessInfo {
        api::ProcessInfo {
            pid,
            name: name.to_string(),
            cmdline: cmdline.iter().map(|arg| arg.to_string()).collect(),
            ..api::ProcessInfo::default()
        }
    }

    fn metrics(pid: u32, cpu_percent: f32, disk_read_bytes: u64) -> api::ProcessMetrics {
        api::ProcessMetrics {
            pid,
            cpu_percent,
            disk_read_bytes,
            ..api::ProcessMetrics::default()
        }
    }

    fn snapshot(etag: u64, infos: Vec<api::ProcessInfo>, metrics: Vec<api::ProcessMetrics>) -> api::Snapshot {
        api::Snapshot {
            machine: api::MachineStats::default(),
            services: api::Tagged { etag: 1, value: Arc::from([]) },
            processes: api::Tagged { etag, value: Arc::from(infos) },
            metrics,
        }
    }

    #[test]
    fn a_passport_keeps_the_first_argument_and_the_console_host() {
        let mut console = info(7, "app.exe", &[r"C:\app.exe", "--flag"]);
        console.console_host_pid = 40;
        let report = Reports::default().report(&snapshot(
            1,
            vec![console, info(8, "idle", &[])],
            vec![metrics(7, 0.0, 0), metrics(8, 0.0, 0)],
        ));

        assert_eq!(&*report.processes[0].first_arg, r"C:\app.exe");
        assert_eq!(report.processes[0].console_host_pid, 40);
        assert_eq!(&*report.processes[1].first_arg, "");
    }

    #[test]
    fn metrics_join_their_passports_by_pid() {
        let report = Reports::default().report(&snapshot(
            1,
            vec![info(7, "a.exe", &[]), info(9, "b.exe", &[])],
            vec![metrics(9, 2.5, 100), metrics(7, 1.0, 50)],
        ));

        let by_pid = |pid: u32| report.processes.iter().find(|r| r.pid == pid).unwrap();
        assert_eq!(&*by_pid(9).name, "b.exe");
        assert_eq!(by_pid(9).cpu_percent, 2.5);
        assert_eq!(by_pid(9).disk_read_bytes, 100);
        assert_eq!(&*by_pid(7).name, "a.exe");
    }

    #[test]
    fn a_metric_without_a_passport_is_dropped_rather_than_named_wrong() {
        let report = Reports::default().report(&snapshot(
            1,
            vec![info(7, "a.exe", &[])],
            vec![metrics(7, 1.0, 0), metrics(99, 5.0, 0)],
        ));

        assert_eq!(report.processes.iter().map(|r| r.pid).collect::<Vec<_>>(), vec![7]);
    }

    #[test]
    fn an_unchanged_process_keeps_its_strings_across_list_changes() {
        let mut reports = Reports::default();
        let first = reports.report(&snapshot(1, vec![info(7, "a.exe", &[])], vec![metrics(7, 0.0, 0)]));
        let second = reports.report(&snapshot(
            2,
            vec![info(7, "a.exe", &[]), info(8, "b.exe", &[])],
            vec![metrics(7, 3.0, 0), metrics(8, 0.0, 0)],
        ));

        assert!(Arc::ptr_eq(&first.processes[0].name, &second.processes[0].name));
        assert_eq!(second.processes[0].cpu_percent, 3.0);
    }

    #[test]
    fn a_reused_pid_gets_the_new_process_strings() {
        let mut reports = Reports::default();
        reports.report(&snapshot(1, vec![info(7, "a.exe", &[])], vec![metrics(7, 0.0, 0)]));
        let reused = reports.report(&snapshot(2, vec![info(7, "b.exe", &[])], vec![metrics(7, 0.0, 0)]));

        assert_eq!(&*reused.processes[0].name, "b.exe");
    }
}
