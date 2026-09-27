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
        exe_path: "".into(),
        package_full_name: "".into(),
        owner: None,
        owner_pid: None,
        category: ProcessCategory::Wsl,
        services: None,
        windows: None,
    }
}

fn home_of(p: &LinuxProcessStats, environments: &[LinuxEnvironmentInfo]) -> Option<usize> {
    environments
        .iter()
        .position(|e| e.mnt_ns == p.mnt_ns && e.pid_ns == p.pid_ns)
        .or_else(|| environments.iter().position(|e| e.pid_ns == p.pid_ns))
}

pub fn environments_from_scan(scan: &RemoteScan, rates: &mut IoRates, now: Instant) -> Vec<WslEnvironment> {
    let mut rows: Vec<ProcessRow> = scan.processes.iter().map(counted_row).collect();
    rates.apply(&mut rows, now);

    let mut homes: Vec<Vec<WslProcess>> = vec![Vec::new(); scan.environments.len() + 1];
    let elsewhere = scan.environments.len();
    for (p, row) in scan.processes.iter().zip(rows) {
        let local_pid = if p.local_pid == 0 { p.global_pid } else { p.local_pid };
        homes[home_of(p, &scan.environments).unwrap_or(elsewhere)].push(WslProcess {
            global_pid: p.global_pid,
            row: ProcessRow { pid: local_pid, ..row },
        });
    }

    let named = scan
        .environments
        .iter()
        .map(|e| (Arc::from(e.name.as_str()), e.kind))
        .chain(std::iter::once((Arc::from(""), EnvironmentKind::Unknown)));
    named
        .zip(homes)
        .filter(|(_, processes)| !processes.is_empty())
        .map(|((name, kind), processes)| WslEnvironment {
            name,
            kind,
            processes: Arc::from(processes),
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
    fn a_service_with_its_own_mounts_stays_in_its_distribution() {
        let report = scan(
            vec![process(100, 1, (1, 1), "init"), process(150, 50, (9, 1), "systemd-resolved")],
            vec![environment((1, 1), EnvironmentKind::CurrentDistro, "Ubuntu")],
        );

        let environments = environments_from_scan(&report, &mut IoRates::default(), Instant::now());

        assert_eq!(layout(&environments), vec![("Ubuntu", vec![(100, 1), (150, 50)])]);
    }

    #[test]
    fn processes_of_no_known_environment_are_gathered_last_and_empty_ones_are_dropped() {
        let report = scan(
            vec![process(2, 2, (7, 7), "kthreadd"), process(100, 1, (1, 1), "init")],
            vec![
                environment((1, 1), EnvironmentKind::CurrentDistro, "Ubuntu"),
                environment((5, 5), EnvironmentKind::DockerContainer, "stopped"),
            ],
        );

        let environments = environments_from_scan(&report, &mut IoRates::default(), Instant::now());

        assert_eq!(layout(&environments), vec![("Ubuntu", vec![(100, 1)]), ("", vec![(2, 2)])]);
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
