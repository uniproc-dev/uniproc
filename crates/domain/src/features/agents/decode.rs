use app_contracts::features::agents::{
    EnvironmentKind, LinuxDockerContainerInfo, LinuxEnvironmentInfo, LinuxMachineStats, LinuxProcessStats,
    LinuxReport,
};

use uniproc_protocol::linux_capnp;

fn text(reader: capnp::text::Reader<'_>) -> capnp::Result<String> {
    Ok(reader.to_str()?.to_string())
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
