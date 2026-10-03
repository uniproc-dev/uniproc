use crate::features::processes::MachineSummary;

#[derive(Clone, PartialEq, Debug)]
pub enum MetricsMsg {
    Machine {
        at: u64,
        cpu: f32,
        memory: f32,
        gpu: f32,
        machine: MachineSummary,
    },
    Rates {
        at: u64,
        disk: u64,
        network: u64,
    },
}
