use crate::features::processes::MachineSummary;
use guinea::prelude::*;

use super::messages::MetricsMsg;

#[derive(Clone, PartialEq, Debug)]
pub struct MetricsState {
    pub cpu_history: Load<Vec<(u64, f32)>>,
    pub memory_history: Load<Vec<(u64, f32)>>,
    pub disk_history: Load<Vec<(u64, f32)>>,
    pub network_history: Load<Vec<(u64, f32)>>,
    pub gpu_history: Load<Vec<(u64, f32)>>,
    pub machine: Load<MachineSummary>,
}

impl Default for MetricsState {
    fn default() -> Self {
        Self {
            cpu_history: Load::Loading,
            memory_history: Load::Loading,
            disk_history: Load::Loading,
            network_history: Load::Loading,
            gpu_history: Load::Loading,
            machine: Load::Loading,
        }
    }
}

#[reducer]
fn metrics(
    this: &mut MetricsState,
    MetricsMsg::SetHistory {
        cpu,
        memory,
        disk,
        network,
        gpu,
        machine,
    }: MetricsMsg,
) {
    this.cpu_history = Load::Ready(cpu);
    this.memory_history = Load::Ready(memory);
    this.disk_history = Load::Ready(disk);
    this.network_history = Load::Ready(network);
    this.gpu_history = Load::Ready(gpu);
    this.machine = Load::Ready(machine);
}
