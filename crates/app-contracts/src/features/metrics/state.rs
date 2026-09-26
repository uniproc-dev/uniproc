use crate::features::processes::MachineSummary;
use guinea::prelude::*;

use super::messages::MetricsMsg;

#[derive(Clone, PartialEq, Debug)]
pub struct MetricsState {
    pub cpu_history: Load<Vec<(u64, f32)>>,
    pub memory_history: Load<Vec<(u64, f32)>>,
    pub machine: Load<MachineSummary>,
}

impl Default for MetricsState {
    fn default() -> Self {
        Self {
            cpu_history: Load::Loading,
            memory_history: Load::Loading,
            machine: Load::Loading,
        }
    }
}

impl Reducer for MetricsState {
    type Update = MetricsMsg;

    fn reduce(&mut self, update: MetricsMsg) {
        match update {
            MetricsMsg::SetHistory {
                cpu,
                memory,
                machine,
            } => {
                self.cpu_history = Load::Ready(cpu);
                self.memory_history = Load::Ready(memory);
                self.machine = Load::Ready(machine);
            }
        }
    }
}
