use std::collections::VecDeque;

use app_contracts::features::agents::{WindowsMachineSample, WindowsReportMessage};
use app_contracts::features::metrics::{MetricsMsg, MetricsState};
use app_contracts::features::processes::MachineSummary;
use guinea::prelude::*;
use guinea_widgets::chart::RingSeries;

type Histories = [Vec<(u64, f32)>; 5];

struct History;

#[expect(non_upper_case_globals)]
impl History {
    const Points: usize = 600;
}

#[derive(Clone, Copy, Debug)]
struct Totals {
    clock_100ns: u64,
    disk_bytes: u64,
    network_bytes: u64,
}

#[expect(non_upper_case_globals)]
impl Totals {
    const TicksPerMs: u64 = 10_000;
    const Window: u64 = 1_000 * Self::TicksPerMs;
}

impl Totals {
    fn went_back(&self, later: &Totals) -> bool {
        later.clock_100ns <= self.clock_100ns
            || later.disk_bytes < self.disk_bytes
            || later.network_bytes < self.network_bytes
    }
}

#[derive(Debug, Default)]
struct Trailing(VecDeque<Totals>);

impl Trailing {
    fn push(&mut self, totals: Totals) -> Option<(u64, u64)> {
        if self.0.back().is_some_and(|last| last.went_back(&totals)) {
            self.0.clear();
        }
        self.0.push_back(totals);
        let start = totals.clock_100ns.saturating_sub(Totals::Window);
        while self.0.get(1).is_some_and(|next| next.clock_100ns <= start) {
            self.0.pop_front();
        }
        let first = self.0.front().filter(|_| self.0.len() > 1)?;
        let elapsed = (totals.clock_100ns - first.clock_100ns) / Totals::TicksPerMs;
        Some((
            per_second(first.disk_bytes, totals.disk_bytes, elapsed),
            per_second(first.network_bytes, totals.network_bytes, elapsed),
        ))
    }

    fn clear(&mut self) {
        self.0.clear();
    }
}

#[derive(Debug)]
pub struct MetricsActor {
    ui_port: Push<MetricsState>,
    cpu_history: RingSeries,
    memory_history: RingSeries,
    disk_history: RingSeries,
    network_history: RingSeries,
    gpu_history: RingSeries,
    totals: Trailing,
    machine: MachineSummary,
    stats: std::cell::RefCell<crate::push_stats::PushStats<(Histories, MachineSummary)>>,
}

impl MetricsActor {
    pub fn new(ui_port: Push<MetricsState>) -> Self {
        Self {
            ui_port,
            cpu_history: RingSeries::new(History::Points),
            memory_history: RingSeries::new(History::Points),
            disk_history: RingSeries::new(History::Points),
            network_history: RingSeries::new(History::Points),
            gpu_history: RingSeries::new(History::Points),
            totals: Trailing::default(),
            machine: MachineSummary::default(),
            stats: std::cell::RefCell::new(crate::push_stats::PushStats::new("metrics")),
        }
    }

    fn publish(&self) {
        let [cpu, memory, disk, network, gpu] = [
            &self.cpu_history,
            &self.memory_history,
            &self.disk_history,
            &self.network_history,
            &self.gpu_history,
        ]
        .map(RingSeries::as_points);
        self.stats.borrow_mut().note((
            [cpu.clone(), memory.clone(), disk.clone(), network.clone(), gpu.clone()],
            self.machine.clone(),
        ));
        self.ui_port.send(MetricsMsg::SetHistory {
            cpu,
            memory,
            disk,
            network,
            gpu,
            machine: self.machine.clone(),
        });
    }
}

fn per_second(before: u64, now: u64, elapsed_ms: u64) -> u64 {
    if elapsed_ms == 0 {
        return 0;
    }
    now.saturating_sub(before).saturating_mul(1_000) / elapsed_ms
}

actor! {
    MetricsActor {
        handlers { WindowsReportMessage, WindowsMachineSample }
    }
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[handler]
fn on_windows_report(this: &mut MetricsActor, msg: WindowsReportMessage) {
    let report = match msg {
        WindowsReportMessage::Report(report) => report,
        WindowsReportMessage::Unavailable(_) => {
            this.totals.clear();
            return;
        }
    };
    let machine = &report.machine;
    let timestamp = now_ms();
    let memory_percent = if machine.total_physical_bytes > 0 {
        (machine.used_physical_bytes() as f32 / machine.total_physical_bytes as f32) * 100.0
    } else {
        0.0
    };

    this.cpu_history.push((timestamp, machine.cpu_percent));
    this.memory_history.push((timestamp, memory_percent));
    this.gpu_history.push((timestamp, machine.gpu_percent()));
    this.machine = MachineSummary {
        cpu_percent: machine.cpu_percent,
        cpu_current_mhz: machine.cpu_current_mhz,
        cpu_max_mhz: machine.cpu_max_mhz,
        memory_used_bytes: machine.used_physical_bytes(),
        memory_total_bytes: machine.total_physical_bytes,
        gpu_percent: machine.gpu_percent(),
        gpu_memory_used_bytes: machine.gpu_dedicated_used_bytes(),
        ..this.machine.clone()
    };
    this.publish();
}

#[handler]
fn on_machine_sample(this: &mut MetricsActor, sample: WindowsMachineSample) {
    let machine = &sample.machine;
    let totals = Totals {
        clock_100ns: sample.clock_100ns,
        disk_bytes: machine.disk_read_bytes + machine.disk_write_bytes,
        network_bytes: machine.net_rx_bytes + machine.net_tx_bytes,
    };
    let Some((disk, network)) = this.totals.push(totals) else {
        return;
    };

    let timestamp = now_ms();
    this.disk_history.push((timestamp, disk as f32));
    this.network_history.push((timestamp, network as f32));
    this.machine.disk_bytes_per_sec = disk;
    this.machine.network_bytes_per_sec = network;
    this.publish();
}

#[cfg(test)]
mod tests {
    use super::{per_second, Totals, Trailing};

    fn at(ms: u64, disk_bytes: u64) -> Totals {
        Totals {
            clock_100ns: ms * Totals::TicksPerMs,
            disk_bytes,
            network_bytes: 0,
        }
    }

    fn disk(trailing: &mut Trailing, ms: u64, bytes: u64) -> Option<u64> {
        trailing.push(at(ms, bytes)).map(|(disk, _)| disk)
    }

    #[test]
    fn a_burst_in_one_tick_counts_over_the_whole_second() {
        let mut trailing = Trailing::default();
        assert_eq!(disk(&mut trailing, 0, 0), None);
        for ms in [200, 400, 600, 800] {
            assert_eq!(disk(&mut trailing, ms, 0), Some(0));
        }
        assert_eq!(disk(&mut trailing, 1_000, 1_000), Some(1_000), "not 5000/s from 1000 bytes in 200 ms");
        assert_eq!(disk(&mut trailing, 1_200, 1_000), Some(1_000), "still inside the last second");
        assert_eq!(disk(&mut trailing, 2_000, 1_000), Some(0), "the burst left the window");
    }

    #[test]
    fn a_steady_flow_reads_the_same_on_every_tick() {
        let mut trailing = Trailing::default();
        disk(&mut trailing, 0, 0);
        for tick in 1..=10u64 {
            assert_eq!(disk(&mut trailing, tick * 200, tick * 100), Some(500));
        }
    }

    #[test]
    fn counters_that_start_over_start_the_window_over() {
        let mut trailing = Trailing::default();
        disk(&mut trailing, 0, 5_000);
        disk(&mut trailing, 200, 6_000);
        assert_eq!(disk(&mut trailing, 400, 100), None);
        assert_eq!(disk(&mut trailing, 600, 300), Some(1_000));
    }

    #[test]
    fn a_rate_is_the_difference_over_the_elapsed_time() {
        assert_eq!(per_second(1_000, 3_000, 500), 4_000);
        assert_eq!(per_second(5_000, 1_000, 500), 0, "a counter that started over is not negative");
        assert_eq!(per_second(0, 1_000, 0), 0);
    }
}
