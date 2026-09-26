use crate::features::processes::MachineSummary;

#[derive(Clone)]
pub enum MetricsMsg {
    SetHistory {
        cpu: Vec<(u64, f32)>,
        memory: Vec<(u64, f32)>,
        machine: MachineSummary,
    },
}
