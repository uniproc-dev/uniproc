use anyhow::anyhow;
use app_contracts::features::agents::{EnvironmentKind, LinuxDockerContainerInfo, LinuxEnvironmentInfo};
use capnp::primitive_list;
use uniproc_protocol::linux_capnp::{
    self, agent_listener, lists_update, machine_sample, process_columns, process_info, transports,
};

use super::linux_report::{
    Columns, Environments, Lists, Machine, MachineCpu, MachineDisk, MachineMemory, Passport, Passports,
    ProcessKey, ProcessSample, Transports, Update,
};

fn text(reader: capnp::text::Reader<'_>) -> capnp::Result<String> {
    Ok(reader.to_str()?.to_string())
}

pub fn update(params: agent_listener::update_params::Reader<'_>) -> anyhow::Result<Update> {
    Ok(Update {
        lists: lists(params.get_lists()?)?,
        columns: columns(params.get_processes()?)?,
        machine: machine(params.get_machine()?)?,
    })
}

fn lists(r: lists_update::Reader<'_>) -> anyhow::Result<Lists> {
    let passports = match r.get_passports().which() {
        Ok(lists_update::passports::Unchanged(())) => Passports::Unchanged,
        Ok(lists_update::passports::Full(list)) => {
            Passports::Full(list?.iter().map(passport).collect::<capnp::Result<_>>()?)
        }
        Ok(lists_update::passports::Delta(delta)) => {
            let delta = delta?;
            Passports::Delta {
                base_etag: delta.get_base_etag(),
                left: delta
                    .get_left()?
                    .iter()
                    .map(|k| ProcessKey {
                        pid: k.get_pid(),
                        sequence_number: k.get_sequence_number(),
                    })
                    .collect(),
                upserted: delta.get_upserted()?.iter().map(passport).collect::<capnp::Result<_>>()?,
            }
        }
        Err(unknown) => return Err(anyhow!("passports carry a member this client does not know: {unknown}")),
    };
    let environments = match r.get_environments().which() {
        Ok(lists_update::environments::Unchanged(())) => None,
        Ok(lists_update::environments::Full(full)) => {
            let full = full?;
            Some(Environments {
                environments: full.get_environments()?.iter().map(environment).collect::<capnp::Result<_>>()?,
                docker_containers: full
                    .get_docker_containers()?
                    .iter()
                    .map(docker_container)
                    .collect::<capnp::Result<_>>()?,
            })
        }
        Err(unknown) => return Err(anyhow!("environments carry a member this client does not know: {unknown}")),
    };
    Ok(Lists {
        passports,
        passport_etag: r.get_passport_etag(),
        environments,
    })
}

fn passport(r: process_info::Reader<'_>) -> capnp::Result<Passport> {
    Ok(Passport {
        key: ProcessKey {
            pid: r.get_pid(),
            sequence_number: r.get_sequence_number(),
        },
        local_pid: r.get_local_pid(),
        mnt_ns: r.get_mnt_ns(),
        pid_ns: r.get_pid_ns(),
        name: text(r.get_name()?)?,
    })
}

fn environment(r: linux_capnp::environment_info::Reader<'_>) -> capnp::Result<LinuxEnvironmentInfo> {
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

fn docker_container(r: linux_capnp::docker_container_info::Reader<'_>) -> capnp::Result<LinuxDockerContainerInfo> {
    Ok(LinuxDockerContainerInfo {
        id: text(r.get_id()?)?,
        mnt_ns: r.get_mnt_ns(),
        pid_ns: r.get_pid_ns(),
        api_version: text(r.get_api_version()?)?,
        raw_json: text(r.get_raw_json()?)?,
    })
}

fn column(list: Option<primitive_list::Reader<'_, u64>>, row: u32) -> Option<u64> {
    list.filter(|l| row < l.len())
        .map(|l| l.get(row))
        .filter(|value| *value != u64::MAX)
}

fn optional<'a>(
    has: bool,
    list: capnp::Result<primitive_list::Reader<'a, u64>>,
) -> capnp::Result<Option<primitive_list::Reader<'a, u64>>> {
    if has { list.map(Some) } else { Ok(None) }
}

fn columns(r: process_columns::Reader<'_>) -> anyhow::Result<Columns> {
    let pids = r.get_pids()?;
    let sequence_numbers = r.get_sequence_numbers()?;
    let cpu_run_time = optional(r.has_cpu_run_time(), r.get_cpu_run_time())?;
    let resident_set = optional(r.has_resident_set(), r.get_resident_set())?;
    let disk_read = optional(r.has_disk_read_bytes(), r.get_disk_read_bytes())?;
    let disk_write = optional(r.has_disk_write_bytes(), r.get_disk_write_bytes())?;
    let pipe_read = optional(r.has_pipe_read_bytes(), r.get_pipe_read_bytes())?;
    let pipe_write = optional(r.has_pipe_write_bytes(), r.get_pipe_write_bytes())?;
    let sendfile = optional(r.has_sendfile_bytes(), r.get_sendfile_bytes())?;
    let transports_list = if r.has_transports() { Some(r.get_transports()?) } else { None };

    let rows = (0..pids.len().min(sequence_numbers.len()))
        .map(|row| ProcessSample {
            key: ProcessKey {
                pid: pids.get(row),
                sequence_number: sequence_numbers.get(row),
            },
            cpu_run_time: column(cpu_run_time, row),
            resident_set: column(resident_set, row).unwrap_or(0),
            disk_read_bytes: column(disk_read, row).unwrap_or(0),
            disk_write_bytes: column(disk_write, row).unwrap_or(0),
            pipe_read_bytes: column(pipe_read, row).unwrap_or(0),
            pipe_write_bytes: column(pipe_write, row).unwrap_or(0),
            sendfile_bytes: column(sendfile, row).unwrap_or(0),
            transports: transports_list
                .filter(|l| row < l.len())
                .map(|l| transport_counts(l.get(row)))
                .unwrap_or_default(),
        })
        .collect();

    Ok(Columns {
        sampled_at: r.get_sampled_at(),
        rows,
    })
}

fn transport_counts(r: transports::Reader<'_>) -> Transports {
    Transports {
        tcp_loopback_rx: r.get_tcp_loopback_rx(),
        tcp_loopback_tx: r.get_tcp_loopback_tx(),
        tcp_remote_rx: r.get_tcp_remote_rx(),
        tcp_remote_tx: r.get_tcp_remote_tx(),
        udp_loopback_rx: r.get_udp_loopback_rx(),
        udp_loopback_tx: r.get_udp_loopback_tx(),
        udp_remote_rx: r.get_udp_remote_rx(),
        udp_remote_tx: r.get_udp_remote_tx(),
        unix_rx: r.get_unix_rx(),
        unix_tx: r.get_unix_tx(),
        vsock_rx: r.get_vsock_rx(),
        vsock_tx: r.get_vsock_tx(),
        p9_rx: r.get_p9_rx(),
        p9_tx: r.get_p9_tx(),
    }
}

fn machine(r: machine_sample::Reader<'_>) -> anyhow::Result<Machine> {
    let cpu = if r.has_cpu() {
        let c = r.get_cpu()?;
        let total = c.get_user_time()
            + c.get_nice_time()
            + c.get_system_time()
            + c.get_idle_time()
            + c.get_iowait_time()
            + c.get_irq_time()
            + c.get_softirq_time()
            + c.get_steal_time();
        Some(MachineCpu {
            busy_time: total.saturating_sub(c.get_idle_time() + c.get_iowait_time()),
            count: c.get_count(),
        })
    } else {
        None
    };
    let memory = if r.has_memory() {
        let m = r.get_memory()?;
        Some(MachineMemory {
            total: m.get_total(),
            free: m.get_free(),
            available: m.get_available(),
            cached: m.get_cached(),
        })
    } else {
        None
    };
    let disk = if r.has_disk() {
        let d = r.get_disk()?;
        Some(MachineDisk {
            read_ops: d.get_read_ops(),
            write_ops: d.get_write_ops(),
            read_bytes: d.get_read_bytes(),
            write_bytes: d.get_write_bytes(),
        })
    } else {
        None
    };
    let network = if r.has_network() {
        Some(transport_counts(r.get_network()?.get_transports()?))
    } else {
        None
    };
    Ok(Machine {
        cpu,
        memory,
        disk,
        network,
    })
}
