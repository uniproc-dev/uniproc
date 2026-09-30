use std::sync::Arc;

use app_contracts::features::agents::{EnvironmentKind, LinuxEnvironmentInfo, LinuxProcessStats, RemoteScan};
use app_contracts::features::processes::{ProcessCategory, ProcessRow, WslEnvironment, WslProcess};
use tokio::time::Instant;

use super::rates::IoRates;

fn counted_row(p: &LinuxProcessStats) -> ProcessRow {
    let name: Arc<str> = Arc::from(p.name.as_str());
    ProcessRow {
        pid: p.global_pid,
        display_name: name.clone(),
        name,
        cpu_percent: p.cpu_percent,
        memory_bytes: p.rss_kb * 1024,
        disk_bytes: p.disk_read_bytes + p.disk_write_bytes,
        net_bytes: p.tcp_rx_remote_bytes
            + p.tcp_tx_remote_bytes
            + p.udp_rx_remote_bytes
            + p.udp_tx_remote_bytes,
        gpu_percent: 0.0,
        gpu_memory_bytes: 0,
        exe_path: "".into(),
        package_full_name: "".into(),
        owner: None,
        owner_pid: None,
        category: ProcessCategory::Wsl,
        services: None,
        windows: None,
        details: Default::default(),
        is_monitor: false,
    }
}

fn names(kind: EnvironmentKind) -> bool {
    matches!(kind, EnvironmentKind::CurrentDistro | EnvironmentKind::DockerContainer)
}

struct Home {
    pid_ns: u64,
    name: Arc<str>,
    kind: EnvironmentKind,
    processes: Vec<WslProcess>,
}

fn home_for(homes: &mut Vec<Home>, pid_ns: u64) -> &mut Home {
    let at = match homes.iter().position(|home| home.pid_ns == pid_ns) {
        Some(at) => at,
        None => {
            homes.push(Home {
                pid_ns,
                name: Arc::from(""),
                kind: EnvironmentKind::Unknown,
                processes: Vec::new(),
            });
            homes.len() - 1
        }
    };
    &mut homes[at]
}

fn name_home(homes: &mut Vec<Home>, environment: &LinuxEnvironmentInfo) {
    let home = home_for(homes, environment.pid_ns);
    if names(environment.kind) && !names(home.kind) {
        home.name = Arc::from(environment.name.as_str());
        home.kind = environment.kind;
    }
}

pub fn environments_from_scan(scan: &RemoteScan, rates: &mut IoRates, now: Instant) -> Vec<WslEnvironment> {
    let mut rows: Vec<ProcessRow> = scan.processes.iter().map(counted_row).collect();
    rates.apply(&mut rows, now);

    let mut homes: Vec<Home> = Vec::new();
    for environment in &scan.environments {
        name_home(&mut homes, environment);
    }
    for (p, row) in scan.processes.iter().zip(rows) {
        let local_pid = if p.local_pid == 0 { p.global_pid } else { p.local_pid };
        home_for(&mut homes, p.pid_ns).processes.push(WslProcess {
            global_pid: p.global_pid,
            row: ProcessRow { pid: local_pid, ..row },
        });
    }

    homes
        .into_iter()
        .filter(|home| !home.processes.is_empty())
        .map(|home| WslEnvironment {
            pid_ns: home.pid_ns,
            name: home.name,
            kind: home.kind,
            processes: Arc::from(home.processes),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn process(global_pid: u32, local_pid: u32, ns: (u64, u64), name: &str) -> LinuxProcessStats {
        LinuxProcessStats {
            global_pid,
            local_pid,
            mnt_ns: ns.0,
            pid_ns: ns.1,
            name: name.into(),
            ..LinuxProcessStats::default()
        }
    }

    fn environment(ns: (u64, u64), kind: EnvironmentKind, name: &str) -> LinuxEnvironmentInfo {
        LinuxEnvironmentInfo {
            mnt_ns: ns.0,
            pid_ns: ns.1,
            kind,
            name: name.into(),
        }
    }

    fn scan(processes: Vec<LinuxProcessStats>, environments: Vec<LinuxEnvironmentInfo>) -> RemoteScan {
        RemoteScan {
            schema_id: "wsl",
            processes,
            machine: Default::default(),
            environments,
            docker_containers: Vec::new(),
        }
    }

    fn layout(environments: &[WslEnvironment]) -> Vec<(&str, Vec<(u32, u32)>)> {
        environments
            .iter()
            .map(|e| {
                let pids = e.processes.iter().map(|p| (p.global_pid, p.row.pid)).collect();
                (&*e.name, pids)
            })
            .collect()
    }

    #[test]
    fn processes_land_in_the_environment_whose_namespaces_they_share() {
        let report = scan(
            vec![
                process(100, 1, (1, 1), "init"),
                process(101, 20, (1, 1), "bash"),
                process(300, 1, (3, 3), "nginx"),
            ],
            vec![
                environment((1, 1), EnvironmentKind::CurrentDistro, "Ubuntu"),
                environment((3, 3), EnvironmentKind::DockerContainer, "web"),
            ],
        );

        let environments = environments_from_scan(&report, &mut IoRates::default(), Instant::now());

        assert_eq!(
            layout(&environments),
            vec![("Ubuntu", vec![(100, 1), (101, 20)]), ("web", vec![(300, 1)])],
            "rows show the pid the environment itself sees, the key stays VM-wide"
        );
        assert_eq!(environments[1].kind, EnvironmentKind::DockerContainer);
    }

    #[test]
    fn services_with_their_own_mounts_stay_in_their_distribution() {
        let report = scan(
            vec![
                process(90, 1, (8, 1), "init"),
                process(100, 2, (1, 1), "systemd"),
                process(150, 50, (9, 1), "systemd-resolved"),
                process(160, 60, (10, 1), "cupsd"),
            ],
            vec![
                environment((8, 1), EnvironmentKind::Unknown, ""),
                environment((1, 1), EnvironmentKind::CurrentDistro, "Ubuntu"),
                environment((9, 1), EnvironmentKind::Unknown, ""),
                environment((10, 1), EnvironmentKind::Unknown, ""),
            ],
        );

        let environments = environments_from_scan(&report, &mut IoRates::default(), Instant::now());

        assert_eq!(
            layout(&environments),
            vec![("Ubuntu", vec![(90, 1), (100, 2), (150, 50), (160, 60)])],
            "the agent reports every private mount namespace; one pid namespace is one environment"
        );
        assert_eq!(environments[0].kind, EnvironmentKind::CurrentDistro);
        assert_eq!(environments[0].pid_ns, 1);
    }

    #[test]
    fn a_pid_namespace_nothing_names_stays_apart_and_empty_ones_are_dropped() {
        let report = scan(
            vec![
                process(2, 2, (7, 7), "kthreadd"),
                process(3, 3, (6, 7), "kworker"),
                process(100, 1, (1, 1), "init"),
            ],
            vec![
                environment((1, 1), EnvironmentKind::CurrentDistro, "Ubuntu"),
                environment((5, 5), EnvironmentKind::DockerContainer, "stopped"),
            ],
        );

        let environments = environments_from_scan(&report, &mut IoRates::default(), Instant::now());

        assert_eq!(
            layout(&environments),
            vec![("Ubuntu", vec![(100, 1)]), ("", vec![(2, 2), (3, 3)])]
        );
        assert_eq!(environments[1].pid_ns, 7);
    }

    #[test]
    fn disk_and_network_are_rates_kept_per_vm_wide_pid() {
        let mut rates = IoRates::default();
        let start = Instant::now();
        let environments = vec![
            environment((1, 1), EnvironmentKind::CurrentDistro, "Ubuntu"),
            environment((3, 3), EnvironmentKind::DockerContainer, "web"),
        ];
        let with_disk = |global_pid, local_pid, ns, disk_read_bytes| LinuxProcessStats {
            disk_read_bytes,
            ..process(global_pid, local_pid, ns, "p")
        };
        environments_from_scan(
            &scan(vec![with_disk(100, 1, (1, 1), 0), with_disk(300, 1, (3, 3), 0)], environments.clone()),
            &mut rates,
            start,
        );

        let later = environments_from_scan(
            &scan(vec![with_disk(100, 1, (1, 1), 1_000), with_disk(300, 1, (3, 3), 4_000)], environments),
            &mut rates,
            start + Duration::from_secs(1),
        );

        let disk: Vec<u64> = later.iter().map(|e| e.processes[0].row.disk_bytes).collect();
        assert_eq!(disk, vec![1_000, 4_000], "two pid 1s in two environments do not share a counter");
    }
}
