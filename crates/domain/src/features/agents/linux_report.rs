use std::collections::HashMap;

use app_contracts::features::agents::{
    LinuxDockerContainerInfo, LinuxEnvironmentInfo, LinuxMachineStats, LinuxProcessStats, LinuxReport,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ProcessKey {
    pub pid: u32,
    pub sequence_number: u64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Passport {
    pub key: ProcessKey,
    pub local_pid: u32,
    pub mnt_ns: u64,
    pub pid_ns: u64,
    pub name: String,
}

#[derive(Debug)]
pub enum Passports {
    Unchanged,
    Full(Vec<Passport>),
    Delta {
        base_etag: u64,
        left: Vec<ProcessKey>,
        upserted: Vec<Passport>,
    },
}

#[derive(Clone, Debug, Default)]
pub struct Environments {
    pub environments: Vec<LinuxEnvironmentInfo>,
    pub docker_containers: Vec<LinuxDockerContainerInfo>,
}

#[derive(Debug)]
pub struct Lists {
    pub passports: Passports,
    pub passport_etag: u64,
    pub environments: Option<Environments>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Transports {
    pub tcp_loopback_rx: u64,
    pub tcp_loopback_tx: u64,
    pub tcp_remote_rx: u64,
    pub tcp_remote_tx: u64,
    pub udp_loopback_rx: u64,
    pub udp_loopback_tx: u64,
    pub udp_remote_rx: u64,
    pub udp_remote_tx: u64,
    pub unix_rx: u64,
    pub unix_tx: u64,
    pub vsock_rx: u64,
    pub vsock_tx: u64,
    pub p9_rx: u64,
    pub p9_tx: u64,
}

#[derive(Clone, Debug, Default)]
pub struct ProcessSample {
    pub key: ProcessKey,
    pub cpu_run_time: Option<u64>,
    pub resident_set: u64,
    pub disk_read_bytes: u64,
    pub disk_write_bytes: u64,
    pub pipe_read_bytes: u64,
    pub pipe_write_bytes: u64,
    pub sendfile_bytes: u64,
    pub transports: Transports,
}

#[derive(Clone, Debug, Default)]
pub struct Columns {
    pub sampled_at: u64,
    pub rows: Vec<ProcessSample>,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct MachineCpu {
    pub busy_time: u64,
    pub count: u32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct MachineMemory {
    pub total: u64,
    pub free: u64,
    pub available: u64,
    pub cached: u64,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct MachineDisk {
    pub read_ops: u64,
    pub write_ops: u64,
    pub read_bytes: u64,
    pub write_bytes: u64,
}

#[derive(Clone, Debug, Default)]
pub struct Machine {
    pub cpu: Option<MachineCpu>,
    pub memory: Option<MachineMemory>,
    pub disk: Option<MachineDisk>,
    pub network: Option<Transports>,
}

#[derive(Debug)]
pub struct Update {
    pub lists: Lists,
    pub columns: Columns,
    pub machine: Machine,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Resync {
    pub held: u64,
    pub base: u64,
}

impl std::fmt::Display for Resync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "a passport delta on etag {} arrived while holding {}", self.base, self.held)
    }
}

#[derive(Default)]
pub struct LinuxReports {
    passports: HashMap<ProcessKey, Passport>,
    passport_etag: u64,
    environments: Environments,
    run_times: HashMap<ProcessKey, u64>,
    sampled_at: Option<u64>,
    cores: u32,
}

struct Kb;

#[expect(non_upper_case_globals)]
impl Kb {
    const Bytes: u64 = 1024;
}

struct Ticks;

#[expect(non_upper_case_globals)]
impl Ticks {
    const Ns: u64 = 100;
}

impl LinuxReports {
    pub fn apply(&mut self, update: Update) -> Result<LinuxReport, Resync> {
        self.apply_lists(update.lists)?;
        if let Some(cpu) = update.machine.cpu {
            self.cores = cpu.count;
        }
        let processes = self.processes(&update.columns);
        Ok(LinuxReport {
            machine: machine(&update.machine),
            processes,
            environments: self.environments.environments.clone(),
            docker_containers: self.environments.docker_containers.clone(),
        })
    }

    fn apply_lists(&mut self, lists: Lists) -> Result<(), Resync> {
        match lists.passports {
            Passports::Unchanged => {}
            Passports::Full(passports) => {
                self.passports = passports.into_iter().map(|p| (p.key, p)).collect();
            }
            Passports::Delta { base_etag, left, upserted } => {
                if base_etag != self.passport_etag {
                    return Err(Resync {
                        held: self.passport_etag,
                        base: base_etag,
                    });
                }
                for key in left {
                    self.passports.remove(&key);
                }
                for passport in upserted {
                    self.passports.insert(passport.key, passport);
                }
            }
        }
        self.passport_etag = lists.passport_etag;
        if let Some(environments) = lists.environments {
            self.environments = environments;
        }
        Ok(())
    }

    fn processes(&mut self, columns: &Columns) -> Vec<LinuxProcessStats> {
        let elapsed = self
            .sampled_at
            .and_then(|before| columns.sampled_at.checked_sub(before))
            .filter(|elapsed| *elapsed > 0);
        let cores = self.cores.max(1) as f64;
        let mut run_times = HashMap::with_capacity(columns.rows.len());
        let processes = columns
            .rows
            .iter()
            .filter_map(|row| {
                let passport = self.passports.get(&row.key)?;
                let cpu_percent = match (row.cpu_run_time, elapsed) {
                    (Some(now), Some(elapsed)) => self
                        .run_times
                        .get(&row.key)
                        .and_then(|before| now.checked_sub(*before))
                        .map(|busy| (busy as f64 / (elapsed as f64 * cores) * 100.0).clamp(0.0, 100.0) as f32)
                        .unwrap_or(0.0),
                    _ => 0.0,
                };
                if let Some(now) = row.cpu_run_time {
                    run_times.insert(row.key, now);
                }
                Some(process(passport, row, cpu_percent))
            })
            .collect();
        self.run_times = run_times;
        self.sampled_at = Some(columns.sampled_at);
        processes
    }
}

fn process(passport: &Passport, row: &ProcessSample, cpu_percent: f32) -> LinuxProcessStats {
    let t = &row.transports;
    LinuxProcessStats {
        global_pid: passport.key.pid,
        local_pid: passport.local_pid,
        mnt_ns: passport.mnt_ns,
        pid_ns: passport.pid_ns,
        name: passport.name.clone(),
        cpu_percent,
        rss_kb: row.resident_set / Kb::Bytes,
        vsock_rx_bytes: t.vsock_rx,
        vsock_tx_bytes: t.vsock_tx,
        p9_rx_bytes: t.p9_rx,
        p9_tx_bytes: t.p9_tx,
        tcp_tx_lo_bytes: t.tcp_loopback_tx,
        tcp_rx_lo_bytes: t.tcp_loopback_rx,
        tcp_tx_remote_bytes: t.tcp_remote_tx,
        tcp_rx_remote_bytes: t.tcp_remote_rx,
        udp_tx_lo_bytes: t.udp_loopback_tx,
        udp_rx_lo_bytes: t.udp_loopback_rx,
        udp_tx_remote_bytes: t.udp_remote_tx,
        udp_rx_remote_bytes: t.udp_remote_rx,
        uds_tx_bytes: t.unix_tx,
        uds_rx_bytes: t.unix_rx,
        disk_read_bytes: row.disk_read_bytes,
        disk_write_bytes: row.disk_write_bytes,
        pipe_read_bytes: row.pipe_read_bytes,
        pipe_write_bytes: row.pipe_write_bytes,
        sendfile_bytes: row.sendfile_bytes,
        ..LinuxProcessStats::default()
    }
}

fn machine(m: &Machine) -> LinuxMachineStats {
    let memory = m.memory.unwrap_or_default();
    let cpu = m.cpu.unwrap_or_default();
    let disk = m.disk.unwrap_or_default();
    let t = m.network.unwrap_or_default();
    LinuxMachineStats {
        total_kb: memory.total / Kb::Bytes,
        free_kb: memory.free / Kb::Bytes,
        available_kb: memory.available / Kb::Bytes,
        used_kb: memory.total.saturating_sub(memory.available) / Kb::Bytes,
        cached_kb: memory.cached / Kb::Bytes,
        busy_ns: cpu.busy_time * Ticks::Ns,
        vsock_rx_bytes: t.vsock_rx,
        vsock_tx_bytes: t.vsock_tx,
        p9_rx_bytes: t.p9_rx,
        p9_tx_bytes: t.p9_tx,
        tcp_tx_lo_bytes: t.tcp_loopback_tx,
        tcp_rx_lo_bytes: t.tcp_loopback_rx,
        tcp_tx_remote_bytes: t.tcp_remote_tx,
        tcp_rx_remote_bytes: t.tcp_remote_rx,
        udp_tx_lo_bytes: t.udp_loopback_tx,
        udp_rx_lo_bytes: t.udp_loopback_rx,
        udp_tx_remote_bytes: t.udp_remote_tx,
        udp_rx_remote_bytes: t.udp_remote_rx,
        uds_tx_bytes: t.unix_tx,
        uds_rx_bytes: t.unix_rx,
        disk_read_bytes: disk.read_bytes,
        disk_write_bytes: disk.write_bytes,
        disk_read_iops: disk.read_ops,
        disk_write_iops: disk.write_ops,
        cpu_count: cpu.count,
        ..LinuxMachineStats::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECOND: u64 = 10_000_000;

    fn key(pid: u32) -> ProcessKey {
        ProcessKey {
            pid,
            sequence_number: u64::from(pid) * 7,
        }
    }

    fn passport(pid: u32, name: &str) -> Passport {
        Passport {
            key: key(pid),
            local_pid: pid,
            name: name.into(),
            ..Passport::default()
        }
    }

    fn sample(pid: u32, run_time: u64) -> ProcessSample {
        ProcessSample {
            key: key(pid),
            cpu_run_time: Some(run_time),
            ..ProcessSample::default()
        }
    }

    fn update(passports: Passports, etag: u64, at: u64, rows: Vec<ProcessSample>) -> Update {
        Update {
            lists: Lists {
                passports,
                passport_etag: etag,
                environments: None,
            },
            columns: Columns { sampled_at: at, rows },
            machine: Machine {
                cpu: Some(MachineCpu { busy_time: 0, count: 4 }),
                ..Machine::default()
            },
        }
    }

    fn names(report: &LinuxReport) -> Vec<&str> {
        report.processes.iter().map(|p| p.name.as_str()).collect()
    }

    #[test]
    fn a_delta_moves_the_passports_the_first_update_gave() {
        let mut reports = LinuxReports::default();
        reports
            .apply(update(
                Passports::Full(vec![passport(1, "init"), passport(2, "bash")]),
                10,
                0,
                vec![sample(1, 0), sample(2, 0)],
            ))
            .unwrap();

        let report = reports
            .apply(update(
                Passports::Delta {
                    base_etag: 10,
                    left: vec![key(2)],
                    upserted: vec![passport(3, "vim")],
                },
                11,
                SECOND,
                vec![sample(1, 0), sample(3, 0)],
            ))
            .unwrap();

        assert_eq!(names(&report), ["init", "vim"]);
    }

    #[test]
    fn a_delta_on_another_etag_asks_for_a_resync() {
        let mut reports = LinuxReports::default();
        reports
            .apply(update(Passports::Full(vec![passport(1, "init")]), 10, 0, vec![]))
            .unwrap();

        let outcome = reports.apply(update(
            Passports::Delta {
                base_etag: 9,
                left: vec![],
                upserted: vec![],
            },
            11,
            SECOND,
            vec![],
        ));

        assert_eq!(outcome.err(), Some(Resync { held: 10, base: 9 }));
    }

    #[test]
    fn cpu_is_run_time_over_elapsed_time_and_cores() {
        let mut reports = LinuxReports::default();
        let first = reports
            .apply(update(Passports::Full(vec![passport(1, "busy")]), 1, 0, vec![sample(1, 0)]))
            .unwrap();
        assert_eq!(first.processes[0].cpu_percent, 0.0, "nothing to compare the first sample to");

        let second = reports
            .apply(update(Passports::Unchanged, 1, SECOND, vec![sample(1, 2 * SECOND)]))
            .unwrap();

        assert!((second.processes[0].cpu_percent - 50.0).abs() < 0.01, "two of four cores for a second");
    }

    #[test]
    fn a_row_without_a_passport_is_left_out() {
        let mut reports = LinuxReports::default();
        let report = reports
            .apply(update(Passports::Full(vec![passport(1, "init")]), 1, 0, vec![sample(1, 0), sample(5, 0)]))
            .unwrap();

        assert_eq!(names(&report), ["init"]);
    }

    #[test]
    fn environments_stay_until_the_agent_sends_new_ones() {
        let mut reports = LinuxReports::default();
        let mut first = update(Passports::Full(vec![]), 1, 0, vec![]);
        first.lists.environments = Some(Environments {
            environments: vec![LinuxEnvironmentInfo {
                name: "Ubuntu".into(),
                ..LinuxEnvironmentInfo::default()
            }],
            docker_containers: vec![],
        });
        reports.apply(first).unwrap();

        let report = reports.apply(update(Passports::Unchanged, 1, SECOND, vec![])).unwrap();

        assert_eq!(report.environments[0].name, "Ubuntu");
    }

    #[test]
    fn machine_memory_and_cpu_come_out_in_the_old_units() {
        let stats = machine(&Machine {
            cpu: Some(MachineCpu {
                busy_time: 3,
                count: 8,
            }),
            memory: Some(MachineMemory {
                total: 8 << 20,
                available: 2 << 20,
                ..MachineMemory::default()
            }),
            ..Machine::default()
        });

        assert_eq!(stats.busy_ns, 300);
        assert_eq!(stats.cpu_count, 8);
        assert_eq!(stats.used_kb, 6 << 10);
    }
}
