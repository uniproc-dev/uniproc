use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

use app_contracts::features::agents::{
    ProcessPriority, SignatureStatus, WindowsAction, WindowsMachineStats, WindowsProcessStats,
    WindowsReport, WindowsServiceState, WindowsServiceStats,
};
use uniproc_windows_agent::api::{self, MachineMetrics, MetricSpec, ProcessMetric};

type Key = (u32, u64);

pub fn spec(interval: Duration) -> MetricSpec {
    MetricSpec {
        interval,
        processes: [
            ProcessMetric::CpuUserTime,
            ProcessMetric::CpuKernelTime,
            ProcessMetric::WorkingSet,
            ProcessMetric::PeakWorkingSet,
            ProcessMetric::PrivateWorkingSet,
            ProcessMetric::Commit,
            ProcessMetric::DiskReadOps,
            ProcessMetric::DiskWriteOps,
            ProcessMetric::DiskReadBytes,
            ProcessMetric::DiskWriteBytes,
            ProcessMetric::NetRxBytes,
            ProcessMetric::NetTxBytes,
        ]
        .into_iter()
        .collect(),
        machine: MachineMetrics::all(),
    }
}

#[derive(Default)]
pub struct Reports {
    passports: HashMap<Key, WindowsProcessStats>,
    services: Vec<WindowsServiceStats>,
    cpu_times: HashMap<Key, u64>,
    machine_cpu: Option<api::MachineCpu>,
}

impl Reports {
    pub fn report(&mut self, update: &api::Update) -> WindowsReport {
        let api::Update { snapshot, sample, changes } = update;
        self.keep_passports(&snapshot.processes.value, changes);
        if changes.full || changes.services {
            self.services = snapshot.services.value.iter().map(service).collect();
        }

        let elapsed = machine_time(self.machine_cpu, sample.machine.cpu);
        let columns = &sample.columns;
        let mut cpu_times = HashMap::with_capacity(sample.pids.len());
        let processes = sample
            .pids
            .iter()
            .zip(sample.sequence_numbers.iter())
            .enumerate()
            .filter_map(|(row, (&pid, &sequence_number))| {
                let key = (pid, sequence_number);
                let cpu_time = read(&columns.cpu_user_time, row)
                    .zip(read(&columns.cpu_kernel_time, row))
                    .map(|(user, kernel)| user.saturating_add(kernel));
                let before = self.cpu_times.get(&key).copied();
                if let Some(cpu_time) = cpu_time {
                    cpu_times.insert(key, cpu_time);
                }
                let passport = self.passports.get(&key)?;
                Some(WindowsProcessStats {
                    cpu_percent: before
                        .zip(cpu_time)
                        .map_or(0.0, |(before, now)| share(now.saturating_sub(before), elapsed)),
                    working_set_bytes: at(&columns.working_set, row),
                    commit_bytes: at(&columns.commit, row),
                    peak_working_set_bytes: at(&columns.peak_working_set, row),
                    private_working_set_bytes: at(&columns.private_working_set, row),
                    disk_read_bytes: at(&columns.disk_read_bytes, row),
                    disk_write_bytes: at(&columns.disk_write_bytes, row),
                    disk_read_iops: at(&columns.disk_read_ops, row),
                    disk_write_iops: at(&columns.disk_write_ops, row),
                    net_rx_bytes: at(&columns.net_rx_bytes, row),
                    net_tx_bytes: at(&columns.net_tx_bytes, row),
                    ..passport.clone()
                })
            })
            .collect();
        self.cpu_times = cpu_times;

        let machine = machine(&sample.machine, self.machine_cpu);
        self.machine_cpu = sample.machine.cpu;

        WindowsReport {
            machine,
            processes,
            services: self.services.clone(),
        }
    }

    fn keep_passports(&mut self, processes: &[api::ProcessInfo], changes: &api::Changes) {
        if changes.full {
            self.passports = processes.iter().map(|info| (key(info), passport(info))).collect();
            return;
        }
        for left in &changes.left {
            self.passports.remove(left);
        }
        if changes.passports.is_empty() {
            return;
        }
        let changed: HashSet<Key> = changes.passports.iter().copied().collect();
        for info in processes.iter().filter(|info| changed.contains(&key(info))) {
            self.passports.insert(key(info), passport(info));
        }
    }
}

fn key(info: &api::ProcessInfo) -> Key {
    (info.pid, info.sequence_number)
}

fn read(column: &Option<Arc<[u64]>>, row: usize) -> Option<u64> {
    column
        .as_deref()
        .and_then(|values| values.get(row))
        .copied()
        .filter(|value| *value != api::NO_DATA_U64)
}

fn at(column: &Option<Arc<[u64]>>, row: usize) -> u64 {
    read(column, row).unwrap_or_default()
}

fn total(cpu: &api::MachineCpu) -> u64 {
    cpu.kernel_time + cpu.user_time
}

fn machine_time(before: Option<api::MachineCpu>, now: Option<api::MachineCpu>) -> u64 {
    match (before, now) {
        (Some(before), Some(now)) => total(&now).saturating_sub(total(&before)),
        _ => 0,
    }
}

fn share(part: u64, whole: u64) -> f32 {
    if whole == 0 {
        return 0.0;
    }
    (part as f64 / whole as f64 * 100.0).min(100.0) as f32
}

fn busy(before: Option<api::MachineCpu>, now: Option<api::MachineCpu>) -> f32 {
    let (Some(before), Some(now)) = (before, now) else {
        return 0.0;
    };
    let whole = total(&now).saturating_sub(total(&before));
    let idle = now.idle_time.saturating_sub(before.idle_time);
    share(whole.saturating_sub(idle), whole)
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

fn machine(sample: &api::MachineSample, before: Option<api::MachineCpu>) -> WindowsMachineStats {
    let cpu = sample.cpu.unwrap_or_default();
    let memory = sample.memory.unwrap_or_default();
    let disk = sample.disk.unwrap_or_default();
    let network = sample.network.unwrap_or_default();
    WindowsMachineStats {
        total_physical_bytes: memory.total_physical,
        available_physical_bytes: memory.available_physical,
        cpu_percent: busy(before, sample.cpu),
        cpu_max_mhz: cpu.max_mhz.into(),
        cpu_current_mhz: cpu.current_mhz.into(),
        disk_read_bytes: disk.read_bytes,
        disk_write_bytes: disk.write_bytes,
        disk_read_iops: disk.read_ops,
        disk_write_iops: disk.write_ops,
        net_rx_bytes: network.rx_bytes,
        net_tx_bytes: network.tx_bytes,
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

    fn sequence(pid: u32) -> u64 {
        u64::from(pid) * 10
    }

    fn info(pid: u32, name: &str, cmdline: &[&str]) -> api::ProcessInfo {
        api::ProcessInfo {
            pid,
            sequence_number: sequence(pid),
            name: name.into(),
            cmdline: cmdline.iter().map(|arg| arg.to_string()).collect(),
            ..api::ProcessInfo::default()
        }
    }

    fn update(snapshot: api::Snapshot, sample: api::Sample, changes: api::Changes) -> api::Update {
        api::Update { snapshot, sample, changes }
    }

    fn full(snapshot: api::Snapshot, sample: api::Sample) -> api::Update {
        update(snapshot, sample, api::Changes { full: true, ..api::Changes::default() })
    }

    fn same(snapshot: api::Snapshot, sample: api::Sample) -> api::Update {
        update(snapshot, sample, api::Changes::default())
    }

    fn key(pid: u32) -> (u32, u64) {
        (pid, sequence(pid))
    }

    fn snapshot(etag: u64, infos: Vec<api::ProcessInfo>) -> api::Snapshot {
        api::Snapshot {
            services: api::Tagged { etag: 1, value: Arc::from([]) },
            processes: api::Tagged { etag, value: Arc::from(infos) },
            states: api::Tagged { etag: 1, value: api::ProcessStates::default() },
        }
    }

    struct Row {
        pid: u32,
        sequence_number: u64,
        cpu_time: u64,
        disk_read_bytes: u64,
    }

    fn row(pid: u32, cpu_time: u64) -> Row {
        Row {
            pid,
            sequence_number: sequence(pid),
            cpu_time,
            disk_read_bytes: 0,
        }
    }

    fn sample(rows: &[Row], machine_time: u64, idle_time: u64) -> api::Sample {
        api::Sample {
            pids: rows.iter().map(|r| r.pid).collect(),
            sequence_numbers: rows.iter().map(|r| r.sequence_number).collect(),
            columns: api::Columns {
                cpu_user_time: Some(rows.iter().map(|r| r.cpu_time).collect()),
                cpu_kernel_time: Some(rows.iter().map(|_| 0).collect()),
                disk_read_bytes: Some(rows.iter().map(|r| r.disk_read_bytes).collect()),
                ..api::Columns::default()
            },
            machine: api::MachineSample {
                cpu: Some(api::MachineCpu {
                    kernel_time: machine_time,
                    idle_time,
                    ..api::MachineCpu::default()
                }),
                ..api::MachineSample::default()
            },
            ..api::Sample::default()
        }
    }

    #[test]
    fn a_passport_keeps_the_first_argument_and_the_console_host() {
        let mut console = info(7, "app.exe", &[r"C:\app.exe", "--flag"]);
        console.console_host_pid = 40;
        let report = Reports::default().report(&full(
            snapshot(1, vec![console, info(8, "idle", &[])]),
            sample(&[row(7, 0), row(8, 0)], 0, 0),
        ));

        assert_eq!(&*report.processes[0].first_arg, r"C:\app.exe");
        assert_eq!(report.processes[0].console_host_pid, 40);
        assert_eq!(&*report.processes[1].first_arg, "");
    }

    #[test]
    fn rows_join_their_passports_by_pid_and_sequence_number() {
        let mut read = row(9, 0);
        read.disk_read_bytes = 100;
        let report = Reports::default().report(&full(
            snapshot(1, vec![info(7, "a.exe", &[]), info(9, "b.exe", &[])]),
            sample(&[read, row(7, 0)], 0, 0),
        ));

        let by_pid = |pid: u32| report.processes.iter().find(|r| r.pid == pid).unwrap();
        assert_eq!(&*by_pid(9).name, "b.exe");
        assert_eq!(by_pid(9).disk_read_bytes, 100);
        assert_eq!(&*by_pid(7).name, "a.exe");
    }

    #[test]
    fn a_row_without_its_passport_is_dropped_rather_than_named_wrong() {
        let mut reused = row(8, 0);
        reused.sequence_number += 1;
        let report = Reports::default().report(&full(
            snapshot(1, vec![info(7, "a.exe", &[]), info(8, "b.exe", &[])]),
            sample(&[row(7, 0), reused, row(99, 0)], 0, 0),
        ));

        assert_eq!(report.processes.iter().map(|r| r.pid).collect::<Vec<_>>(), vec![7]);
    }

    #[test]
    fn an_unchanged_process_keeps_its_strings_across_list_changes() {
        let mut reports = Reports::default();
        let first = reports.report(&full(snapshot(1, vec![info(7, "a.exe", &[])]), sample(&[row(7, 0)], 0, 0)));
        let second = reports.report(&update(
            snapshot(2, vec![info(7, "a.exe", &[]), info(8, "b.exe", &[])]),
            sample(&[row(7, 0), row(8, 0)], 0, 0),
            api::Changes { passports: vec![key(8)], states: vec![key(8)], ..api::Changes::default() },
        ));

        assert!(Arc::ptr_eq(&first.processes[0].name, &second.processes[0].name));
        assert_eq!(&*second.processes[1].name, "b.exe");
    }

    #[test]
    fn a_changed_passport_is_read_again() {
        let mut reports = Reports::default();
        reports.report(&full(snapshot(1, vec![info(7, "a.exe", &[])]), sample(&[row(7, 0)], 0, 0)));
        let mut enriched = info(7, "a.exe", &[]);
        enriched.display_name = "App".into();
        let report = reports.report(&update(
            snapshot(2, vec![enriched]),
            sample(&[row(7, 0)], 0, 0),
            api::Changes { passports: vec![key(7)], ..api::Changes::default() },
        ));

        assert_eq!(&*report.processes[0].display_name, "App");
    }

    #[test]
    fn a_reused_pid_gets_the_new_process_strings() {
        let mut reports = Reports::default();
        reports.report(&full(snapshot(1, vec![info(7, "a.exe", &[])]), sample(&[row(7, 0)], 0, 0)));
        let mut reborn = info(7, "b.exe", &[]);
        reborn.sequence_number += 1;
        let mut reused = row(7, 0);
        reused.sequence_number += 1;
        let report = reports.report(&update(
            snapshot(2, vec![reborn]),
            sample(&[reused], 0, 0),
            api::Changes {
                passports: vec![(7, sequence(7) + 1)],
                left: vec![key(7)],
                ..api::Changes::default()
            },
        ));

        assert_eq!(&*report.processes[0].name, "b.exe");
        assert_eq!(reports.passports.len(), 1);
    }

    #[test]
    fn services_are_read_again_only_when_they_changed() {
        let mut reports = Reports::default();
        let mut list = snapshot(1, vec![]);
        list.services.value = Arc::from([api::ServiceStats { name: "svc".into(), ..api::ServiceStats::default() }]);
        let first = reports.report(&full(list.clone(), sample(&[], 0, 0)));
        let kept = reports.report(&same(list.clone(), sample(&[], 0, 0)));
        list.services.value = Arc::from([]);
        let changed = reports.report(&update(
            list,
            sample(&[], 0, 0),
            api::Changes { services: true, ..api::Changes::default() },
        ));

        assert_eq!(first.services.len(), 1);
        assert!(Arc::ptr_eq(&first.services[0].name, &kept.services[0].name));
        assert!(changed.services.is_empty());
    }

    #[test]
    fn cpu_is_the_share_of_machine_time_between_two_samples() {
        let mut reports = Reports::default();
        let list = snapshot(1, vec![info(7, "a.exe", &[])]);
        let first = reports.report(&full(list.clone(), sample(&[row(7, 1_000)], 10_000, 4_000)));
        let second = reports.report(&same(list, sample(&[row(7, 1_250)], 11_000, 4_600)));

        assert_eq!(first.processes[0].cpu_percent, 0.0);
        assert_eq!(first.machine.cpu_percent, 0.0);
        assert_eq!(second.processes[0].cpu_percent, 25.0);
        assert_eq!(second.machine.cpu_percent, 40.0);
    }

    #[test]
    fn a_row_marked_as_no_data_reads_as_nothing_rather_than_the_maximum() {
        let mut reports = Reports::default();
        let list = snapshot(1, vec![info(7, "a.exe", &[])]);
        let mut unread = sample(&[row(7, api::NO_DATA_U64)], 10_000, 0);
        unread.columns.working_set = Some(Arc::from([api::NO_DATA_U64]));
        let first = reports.report(&full(list.clone(), unread));
        let second = reports.report(&same(list, sample(&[row(7, 500)], 11_000, 0)));

        assert_eq!(first.processes[0].working_set_bytes, 0);
        assert_eq!(second.processes[0].cpu_percent, 0.0);
    }

    #[test]
    fn a_process_seen_for_the_first_time_has_no_cpu_yet() {
        let mut reports = Reports::default();
        reports.report(&full(snapshot(1, vec![info(7, "a.exe", &[])]), sample(&[row(7, 0)], 10_000, 0)));
        let report = reports.report(&update(
            snapshot(2, vec![info(7, "a.exe", &[]), info(8, "b.exe", &[])]),
            sample(&[row(7, 0), row(8, 900_000)], 11_000, 0),
            api::Changes { passports: vec![key(8)], states: vec![key(8)], ..api::Changes::default() },
        ));

        let by_pid = |pid: u32| report.processes.iter().find(|r| r.pid == pid).unwrap();
        assert_eq!(by_pid(8).cpu_percent, 0.0);
    }
}
