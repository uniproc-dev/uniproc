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
}

#[derive(Debug)]
pub struct MetricsActor {
    ui_port: Push<MetricsState>,
    cpu_history: RingSeries,
    memory_history: RingSeries,
    disk_history: RingSeries,
    network_history: RingSeries,
    gpu_history: RingSeries,
    totals: Option<Totals>,
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
            totals: None,
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
            this.totals = None;
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
    let (disk, network) = match this.totals.replace(totals) {
        Some(before) => {
            let elapsed = totals.clock_100ns.saturating_sub(before.clock_100ns) / Totals::TicksPerMs;
            (
                per_second(before.disk_bytes, totals.disk_bytes, elapsed),
                per_second(before.network_bytes, totals.network_bytes, elapsed),
            )
        }
        None => return,
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
    use super::per_second;

    #[test]
    fn a_rate_is_the_difference_over_the_elapsed_time() {
        assert_eq!(per_second(1_000, 3_000, 500), 4_000);
        assert_eq!(per_second(5_000, 1_000, 500), 0, "a counter that started over is not negative");
        assert_eq!(per_second(0, 1_000, 0), 0);
    }
}
