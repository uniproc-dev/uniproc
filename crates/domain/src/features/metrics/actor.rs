use app_contracts::features::agents::WindowsReportMessage;
use app_contracts::features::metrics::{MetricsMsg, MetricsState};
use app_contracts::features::processes::MachineSummary;
use guinea::prelude::*;
use guinea_widgets::chart::RingSeries;

#[derive(Debug)]
pub struct MetricsActor {
    ui_port: Push<MetricsState>,
    cpu_history: RingSeries,
    memory_history: RingSeries,
    machine: MachineSummary,
    stats: std::cell::RefCell<crate::push_stats::PushStats<(Vec<(u64, f32)>, Vec<(u64, f32)>, MachineSummary)>>,
}

impl MetricsActor {
    pub fn new(ui_port: Push<MetricsState>) -> Self {
        Self {
            ui_port,
            cpu_history: RingSeries::new(120),
            memory_history: RingSeries::new(120),
            machine: MachineSummary::default(),
            stats: std::cell::RefCell::new(crate::push_stats::PushStats::new("metrics")),
        }
    }

    fn publish(&self) {
        let cpu = self.cpu_history.as_points();
        let memory = self.memory_history.as_points();
        self.stats
            .borrow_mut()
            .note((cpu.clone(), memory.clone(), self.machine.clone()));
        self.ui_port.send(MetricsMsg::SetHistory {
            cpu,
            memory,
            machine: self.machine.clone(),
        });
    }
}

actor! {
    MetricsActor {
        handlers { WindowsReportMessage }
    }
}

#[handler]
fn on_windows_report(this: &mut MetricsActor, msg: WindowsReportMessage) {
    let WindowsReportMessage::Report(report) = msg else {
        return;
    };
    let machine = &report.machine;
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let memory_percent = if machine.total_physical_kb > 0 {
        (machine.used_physical_kb as f32 / machine.total_physical_kb as f32) * 100.0
    } else {
        0.0
    };
    this.cpu_history.push((timestamp, machine.cpu_percent));
    this.memory_history.push((timestamp, memory_percent));
    this.machine = MachineSummary {
        cpu_percent: machine.cpu_percent,
        cpu_current_mhz: machine.cpu_current_mhz,
        cpu_max_mhz: machine.cpu_max_mhz,
        memory_used_bytes: machine.used_physical_kb * 1024,
        memory_total_bytes: machine.total_physical_kb * 1024,
    };
    this.publish();
}
