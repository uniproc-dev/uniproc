use app_contracts::features::agents::{
    EnvironmentKind, LinuxDockerContainerInfo, LinuxEnvironmentInfo, LinuxMachineStats, LinuxProcessStats,
    LinuxReport, ProcessPriority, SignatureStatus, WindowsMachineStats, WindowsProcessStats,
    WindowsServiceState, WindowsServiceStats,
};
use std::collections::HashMap;
use std::sync::Arc;

use uniproc_protocol::linux_capnp;
use uniproc_protocol::windows_capnp;

fn text(reader: capnp::text::Reader<'_>) -> capnp::Result<String> {
    Ok(reader.to_str()?.to_string())
}

fn shared(reader: capnp::text::Reader<'_>) -> capnp::Result<Arc<str>> {
    Ok(Arc::from(reader.to_str()?))
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProcessInfo {
    pub pid: u32,
    pub parent_pid: u32,
    pub session_id: u32,
    pub name: Arc<str>,
    pub first_arg: Arc<str>,
    pub package_full_name: Arc<str>,
    pub package_relative_app_id: Arc<str>,
    pub is_service: bool,
    pub is_kernel_process: bool,
    pub is_windows_process: bool,
    pub signature: SignatureStatus,
    pub image_path: Arc<str>,
    pub display_name: Arc<str>,
    pub console_host_pid: u32,
}

pub struct Processes {
    infos: Vec<ProcessInfo>,
    by_pid: HashMap<u32, usize>,
}

impl Processes {
    pub fn new(infos: Vec<ProcessInfo>) -> Self {
        let by_pid = infos
            .iter()
            .enumerate()
            .map(|(at, info)| (info.pid, at))
            .collect();
        Self { infos, by_pid }
    }

    pub fn len(&self) -> usize {
        self.infos.len()
    }

    pub fn is_empty(&self) -> bool {
        self.infos.is_empty()
    }
}

pub fn windows_processes(
    list: capnp::struct_list::Reader<'_, windows_capnp::process_info::Owned>,
) -> capnp::Result<Processes> {
    let infos = list
        .iter()
        .map(windows_process_info)
        .collect::<capnp::Result<Vec<_>>>()?;
    Ok(Processes::new(infos))
}

pub fn windows_services(
    list: capnp::struct_list::Reader<'_, windows_capnp::service_stats::Owned>,
) -> capnp::Result<Vec<WindowsServiceStats>> {
    list.iter().map(windows_service).collect()
}

pub fn join_metrics(
    processes: &Processes,
    metrics: capnp::struct_list::Reader<'_, windows_capnp::process_metrics::Owned>,
) -> Vec<WindowsProcessStats> {
    metrics
        .iter()
        .filter_map(|m| {
            let info = &processes.infos[*processes.by_pid.get(&m.get_pid())?];
            Some(WindowsProcessStats {
                pid: info.pid,
                parent_pid: info.parent_pid,
                session_id: info.session_id,
                name: info.name.clone(),
                first_arg: info.first_arg.clone(),
                package_full_name: info.package_full_name.clone(),
                package_relative_app_id: info.package_relative_app_id.clone(),
                cpu_percent: m.get_cpu_percent(),
                working_set_kb: m.get_working_set_kb(),
                private_bytes_kb: m.get_private_bytes_kb(),
                peak_working_set_kb: m.get_peak_working_set_kb(),
                private_working_set_kb: m.get_private_working_set_kb(),
                disk_read_bytes: m.get_disk_read_bytes(),
                disk_write_bytes: m.get_disk_write_bytes(),
                disk_read_iops: m.get_disk_read_iops(),
                disk_write_iops: m.get_disk_write_iops(),
                net_rx_bytes: m.get_net_rx_bytes(),
                net_tx_bytes: m.get_net_tx_bytes(),
                is_service: info.is_service,
                is_kernel_process: info.is_kernel_process,
                is_windows_process: info.is_windows_process,
                signature: info.signature,
                image_path: info.image_path.clone(),
                display_name: info.display_name.clone(),
                console_host_pid: info.console_host_pid,
            })
        })
        .collect()
}

fn service_state(raw: windows_capnp::ServiceState) -> WindowsServiceState {
    match raw {
        windows_capnp::ServiceState::Unknown => WindowsServiceState::Unknown,
        windows_capnp::ServiceState::Stopped => WindowsServiceState::Stopped,
        windows_capnp::ServiceState::StartPending => WindowsServiceState::StartPending,
        windows_capnp::ServiceState::StopPending => WindowsServiceState::StopPending,
        windows_capnp::ServiceState::Running => WindowsServiceState::Running,
        windows_capnp::ServiceState::ContinuePending => WindowsServiceState::ContinuePending,
        windows_capnp::ServiceState::PausePending => WindowsServiceState::PausePending,
        windows_capnp::ServiceState::Paused => WindowsServiceState::Paused,
    }
}

fn windows_service(r: windows_capnp::service_stats::Reader<'_>) -> capnp::Result<WindowsServiceStats> {
    let state = match r.get_state() {
        Ok(raw) => service_state(raw),
        Err(err) => {
            tracing::warn!(?err, "agent reported a service state this build does not know");
            WindowsServiceState::Unknown
        }
    };

    Ok(WindowsServiceStats {
        name: shared(r.get_name()?)?,
        display_name: shared(r.get_display_name()?)?,
        pid: r.get_pid(),
        state,
        load_group: shared(r.get_load_group()?)?,
        description: shared(r.get_description()?)?,
        image_path: shared(r.get_image_path()?)?,
    })
}

pub fn windows_machine(r: windows_capnp::machine_stats::Reader<'_>) -> capnp::Result<WindowsMachineStats> {
    Ok(WindowsMachineStats {
        total_physical_kb: r.get_total_physical_kb(),
        available_physical_kb: r.get_available_physical_kb(),
        used_physical_kb: r.get_used_physical_kb(),
        cpu_percent: r.get_cpu_percent(),
        cpu_max_mhz: r.get_cpu_max_mhz(),
        cpu_current_mhz: r.get_cpu_current_mhz(),
        disk_read_bytes: r.get_disk_read_bytes(),
        disk_write_bytes: r.get_disk_write_bytes(),
        disk_read_iops: r.get_disk_read_iops(),
        disk_write_iops: r.get_disk_write_iops(),
        net_rx_bytes: r.get_net_rx_bytes(),
        net_tx_bytes: r.get_net_tx_bytes(),
    })
}

fn windows_process_info(r: windows_capnp::process_info::Reader<'_>) -> capnp::Result<ProcessInfo> {
    let cmdline = r.get_cmdline()?;
    let first_arg = if cmdline.is_empty() {
        Arc::from("")
    } else {
        shared(cmdline.get(0)?)?
    };

    Ok(ProcessInfo {
        pid: r.get_pid(),
        parent_pid: r.get_parent_pid(),
        session_id: r.get_session_id(),
        name: shared(r.get_name()?)?,
        first_arg,
        package_full_name: shared(r.get_package_full_name()?)?,
        package_relative_app_id: shared(r.get_package_relative_app_id()?)?,
        is_service: r.get_is_service(),
        is_kernel_process: r.get_is_kernel_process(),
        is_windows_process: r.get_is_windows_process(),
        signature: r.get_signature().map(signature).unwrap_or_default(),
        image_path: shared(r.get_image_path()?)?,
        display_name: shared(r.get_display_name()?)?,
        console_host_pid: r.get_console_host_pid(),
    })
}

fn signature(status: windows_capnp::SignatureStatus) -> SignatureStatus {
    match status {
        windows_capnp::SignatureStatus::Unknown => SignatureStatus::Unknown,
        windows_capnp::SignatureStatus::Unsigned => SignatureStatus::Unsigned,
        windows_capnp::SignatureStatus::Microsoft => SignatureStatus::Microsoft,
        windows_capnp::SignatureStatus::ThirdParty => SignatureStatus::ThirdParty,
    }
}

pub fn priority(priority: ProcessPriority) -> windows_capnp::ProcessPriority {
    match priority {
        ProcessPriority::Idle => windows_capnp::ProcessPriority::Idle,
        ProcessPriority::BelowNormal => windows_capnp::ProcessPriority::BelowNormal,
        ProcessPriority::Normal => windows_capnp::ProcessPriority::Normal,
        ProcessPriority::AboveNormal => windows_capnp::ProcessPriority::AboveNormal,
        ProcessPriority::High => windows_capnp::ProcessPriority::High,
        ProcessPriority::Realtime => windows_capnp::ProcessPriority::Realtime,
    }
}

pub fn linux_report(reader: linux_capnp::report::Reader<'_>) -> capnp::Result<LinuxReport> {
    let processes = reader
        .get_processes()?
        .iter()
        .map(linux_process)
        .collect::<capnp::Result<Vec<_>>>()?;
    let environments = reader
        .get_environments()?
        .iter()
        .map(linux_environment)
        .collect::<capnp::Result<Vec<_>>>()?;
    let docker_containers = reader
        .get_docker_containers()?
        .iter()
        .map(linux_docker_container)
        .collect::<capnp::Result<Vec<_>>>()?;

    Ok(LinuxReport {
        machine: linux_machine(reader.get_machine()?)?,
        processes,
        environments,
        docker_containers,
    })
}

fn linux_machine(r: linux_capnp::machine_stats::Reader<'_>) -> capnp::Result<LinuxMachineStats> {
    Ok(LinuxMachineStats {
        total_kb: r.get_total_kb(),
        free_kb: r.get_free_kb(),
        available_kb: r.get_available_kb(),
        used_kb: r.get_used_kb(),
        cached_kb: r.get_cached_kb(),
        busy_ns: r.get_busy_ns(),
        last_tsc: r.get_last_tsc(),
        vsock_rx_bytes: r.get_vsock_rx_bytes(),
        vsock_tx_bytes: r.get_vsock_tx_bytes(),
        p9_rx_bytes: r.get_p9_rx_bytes(),
        p9_tx_bytes: r.get_p9_tx_bytes(),
        tcp_tx_lo_bytes: r.get_tcp_tx_lo_bytes(),
        tcp_rx_lo_bytes: r.get_tcp_rx_lo_bytes(),
        tcp_tx_remote_bytes: r.get_tcp_tx_remote_bytes(),
        tcp_rx_remote_bytes: r.get_tcp_rx_remote_bytes(),
        udp_tx_lo_bytes: r.get_udp_tx_lo_bytes(),
        udp_rx_lo_bytes: r.get_udp_rx_lo_bytes(),
        udp_tx_remote_bytes: r.get_udp_tx_remote_bytes(),
        udp_rx_remote_bytes: r.get_udp_rx_remote_bytes(),
        uds_tx_bytes: r.get_uds_tx_bytes(),
        uds_rx_bytes: r.get_uds_rx_bytes(),
        disk_read_bytes: r.get_disk_read_bytes(),
        disk_write_bytes: r.get_disk_write_bytes(),
        disk_read_iops: r.get_disk_read_iops(),
        disk_write_iops: r.get_disk_write_iops(),
        pipe_read_bytes: r.get_pipe_read_bytes(),
        pipe_write_bytes: r.get_pipe_write_bytes(),
        sendfile_bytes: r.get_sendfile_bytes(),
        cpu_count: r.get_cpu_count(),
    })
}

fn linux_process(r: linux_capnp::process_stats::Reader<'_>) -> capnp::Result<LinuxProcessStats> {
    Ok(LinuxProcessStats {
        global_pid: r.get_global_pid(),
        local_pid: r.get_local_pid(),
        mnt_ns: r.get_mnt_ns(),
        pid_ns: r.get_pid_ns(),
        name: text(r.get_name()?)?,
        cpu_percent: r.get_cpu_percent(),
        rss_kb: r.get_rss_kb(),
        last_active_ns: r.get_last_active_ns(),
        vsock_rx_bytes: r.get_vsock_rx_bytes(),
        vsock_tx_bytes: r.get_vsock_tx_bytes(),
        p9_rx_bytes: r.get_p9_rx_bytes(),
        p9_tx_bytes: r.get_p9_tx_bytes(),
        tcp_tx_lo_bytes: r.get_tcp_tx_lo_bytes(),
        tcp_rx_lo_bytes: r.get_tcp_rx_lo_bytes(),
        tcp_tx_remote_bytes: r.get_tcp_tx_remote_bytes(),
        tcp_rx_remote_bytes: r.get_tcp_rx_remote_bytes(),
        udp_tx_lo_bytes: r.get_udp_tx_lo_bytes(),
        udp_rx_lo_bytes: r.get_udp_rx_lo_bytes(),
        udp_tx_remote_bytes: r.get_udp_tx_remote_bytes(),
        udp_rx_remote_bytes: r.get_udp_rx_remote_bytes(),
        uds_tx_bytes: r.get_uds_tx_bytes(),
        uds_rx_bytes: r.get_uds_rx_bytes(),
        disk_read_bytes: r.get_disk_read_bytes(),
        disk_write_bytes: r.get_disk_write_bytes(),
        disk_read_iops: r.get_disk_read_iops(),
        disk_write_iops: r.get_disk_write_iops(),
        pipe_read_bytes: r.get_pipe_read_bytes(),
        pipe_write_bytes: r.get_pipe_write_bytes(),
        sendfile_bytes: r.get_sendfile_bytes(),
    })
}

fn linux_environment(r: linux_capnp::environment_info::Reader<'_>) -> capnp::Result<LinuxEnvironmentInfo> {
    Ok(LinuxEnvironmentInfo {
        mnt_ns: r.get_mnt_ns(),
        pid_ns: r.get_pid_ns(),
        kind: r.get_kind().map(environment_kind).unwrap_or_default(),
        name: text(r.get_name()?)?,
    })
}

fn environment_kind(kind: linux_capnp::EnvironmentKind) -> EnvironmentKind {
    match kind {
        linux_capnp::EnvironmentKind::Unknown => EnvironmentKind::Unknown,
        linux_capnp::EnvironmentKind::CurrentDistro => EnvironmentKind::CurrentDistro,
        linux_capnp::EnvironmentKind::DockerContainer => EnvironmentKind::DockerContainer,
        linux_capnp::EnvironmentKind::UnknownExternalNamespace => EnvironmentKind::UnknownExternalNamespace,
    }
}

fn linux_docker_container(
    r: linux_capnp::docker_container_info::Reader<'_>,
) -> capnp::Result<LinuxDockerContainerInfo> {
    Ok(LinuxDockerContainerInfo {
        id: text(r.get_id()?)?,
        mnt_ns: r.get_mnt_ns(),
        pid_ns: r.get_pid_ns(),
        api_version: text(r.get_api_version()?)?,
        raw_json: text(r.get_raw_json()?)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use capnp::message::{Builder, HeapAllocator};
    use uniproc_protocol::windows_capnp::windows_agent;

    struct Process<'a> {
        pid: u32,
        name: &'a str,
        args: &'a [&'a str],
        console_host_pid: u32,
    }

    fn encode_processes(processes: &[Process<'_>]) -> Builder<HeapAllocator> {
        let mut message = Builder::new_default();
        let root = message.init_root::<windows_agent::get_processes_results::Builder<'_>>();
        let mut list = root.init_processes(processes.len() as u32);
        for (at, process) in processes.iter().enumerate() {
            let mut entry = list.reborrow().get(at as u32);
            entry.set_pid(process.pid);
            entry.set_name(process.name);
            entry.set_console_host_pid(process.console_host_pid);
            let mut args = entry.init_cmdline(process.args.len() as u32);
            for (index, arg) in process.args.iter().enumerate() {
                args.set(index as u32, *arg);
            }
        }
        message
    }

    fn encode_metrics(metrics: &[(u32, f32, u64)]) -> Builder<HeapAllocator> {
        let mut message = Builder::new_default();
        let root = message.init_root::<windows_agent::get_process_metrics_results::Builder<'_>>();
        let mut list = root.init_metrics(metrics.len() as u32);
        for (at, (pid, cpu, disk)) in metrics.iter().enumerate() {
            let mut entry = list.reborrow().get(at as u32);
            entry.set_pid(*pid);
            entry.set_cpu_percent(*cpu);
            entry.set_disk_read_bytes(*disk);
        }
        message
    }

    fn processes(message: &Builder<HeapAllocator>) -> Processes {
        let root = message
            .get_root_as_reader::<windows_agent::get_processes_results::Reader<'_>>()
            .expect("processes root");
        windows_processes(root.get_processes().expect("list")).expect("decode")
    }

    fn joined(processes: &Processes, message: &Builder<HeapAllocator>) -> Vec<WindowsProcessStats> {
        let root = message
            .get_root_as_reader::<windows_agent::get_process_metrics_results::Reader<'_>>()
            .expect("metrics root");
        join_metrics(processes, root.get_metrics().expect("list"))
    }

    #[test]
    fn a_passport_keeps_the_first_argument_and_the_console_host() {
        let decoded = processes(&encode_processes(&[
            Process { pid: 7, name: "app.exe", args: &[r"C:\app.exe", "--flag"], console_host_pid: 40 },
            Process { pid: 8, name: "idle", args: &[], console_host_pid: 0 },
        ]));

        assert_eq!(&*decoded.infos[0].first_arg, r"C:\app.exe");
        assert_eq!(decoded.infos[0].console_host_pid, 40);
        assert_eq!(&*decoded.infos[1].first_arg, "");
    }

    #[test]
    fn metrics_join_their_passports_by_pid() {
        let passports = processes(&encode_processes(&[
            Process { pid: 7, name: "a.exe", args: &[], console_host_pid: 0 },
            Process { pid: 9, name: "b.exe", args: &[], console_host_pid: 0 },
        ]));
        let rows = joined(&passports, &encode_metrics(&[(9, 2.5, 100), (7, 1.0, 50)]));

        let by_pid = |pid: u32| rows.iter().find(|r| r.pid == pid).unwrap();
        assert_eq!(&*by_pid(9).name, "b.exe");
        assert_eq!(by_pid(9).cpu_percent, 2.5);
        assert_eq!(by_pid(9).disk_read_bytes, 100);
        assert_eq!(&*by_pid(7).name, "a.exe");
    }

    #[test]
    fn a_metric_without_a_passport_is_dropped_rather_than_named_wrong() {
        let passports = processes(&encode_processes(&[Process { pid: 7, name: "a.exe", args: &[], console_host_pid: 0 }]));
        let rows = joined(&passports, &encode_metrics(&[(7, 1.0, 0), (99, 5.0, 0)]));

        assert_eq!(rows.iter().map(|r| r.pid).collect::<Vec<_>>(), vec![7]);
    }
}
