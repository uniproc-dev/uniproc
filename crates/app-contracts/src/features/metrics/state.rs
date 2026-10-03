use std::collections::VecDeque;

use crate::features::processes::MachineSummary;
use guinea::prelude::*;

use super::messages::MetricsMsg;

#[derive(Clone, PartialEq, Debug, Default)]
pub struct History(VecDeque<(u64, f32)>);

#[expect(non_upper_case_globals)]
impl History {
    pub const Points: usize = 600;
}

impl History {
    pub fn push(&mut self, point: (u64, f32)) {
        if self.0.len() == Self::Points {
            self.0.pop_front();
        }
        self.0.push_back(point);
    }

    pub fn last(&self) -> Option<(u64, f32)> {
        self.0.back().copied()
    }

    pub fn iter(&self) -> std::collections::vec_deque::Iter<'_, (u64, f32)> {
        self.0.iter()
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct MetricsState {
    pub cpu_history: Load<History>,
    pub memory_history: Load<History>,
    pub disk_history: Load<History>,
    pub network_history: Load<History>,
    pub gpu_history: Load<History>,
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

fn ready<T: Default>(load: &mut Load<T>) -> &mut T {
    if !matches!(load, Load::Ready(_)) {
        *load = Load::Ready(T::default());
    }
    match load {
        Load::Ready(value) => value,
        _ => unreachable!(),
    }
}

#[reducer]
fn metrics(this: &mut MetricsState, msg: MetricsMsg) {
    for history in [
        &mut this.cpu_history,
        &mut this.memory_history,
        &mut this.disk_history,
        &mut this.network_history,
        &mut this.gpu_history,
    ] {
        ready(history);
    }
    match msg {
        MetricsMsg::Machine {
            at,
            cpu,
            memory,
            gpu,
            machine,
        } => {
            ready(&mut this.cpu_history).push((at, cpu));
            ready(&mut this.memory_history).push((at, memory));
            ready(&mut this.gpu_history).push((at, gpu));
            let shown = ready(&mut this.machine);
            *shown = MachineSummary {
                disk_bytes_per_sec: shown.disk_bytes_per_sec,
                network_bytes_per_sec: shown.network_bytes_per_sec,
                ..machine
            };
        }
        MetricsMsg::Rates { at, disk, network } => {
            ready(&mut this.disk_history).push((at, disk as f32));
            ready(&mut this.network_history).push((at, network as f32));
            let shown = ready(&mut this.machine);
            shown.disk_bytes_per_sec = disk;
            shown.network_bytes_per_sec = network;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_history_keeps_the_newest_points() {
        let mut history = History::default();
        for at in 0..=History::Points as u64 {
            history.push((at, at as f32));
        }

        assert_eq!(history.iter().count(), History::Points);
        assert_eq!(history.iter().next(), Some(&(1, 1.0)));
        assert_eq!(history.last(), Some((History::Points as u64, History::Points as f32)));
    }

    #[test]
    fn a_rate_keeps_the_machine_and_a_machine_keeps_the_rates() {
        let mut state = MetricsState::default();
        let machine = MachineSummary {
            cpu_percent: 12.0,
            ..MachineSummary::default()
        };
        state.reduce(MetricsMsg::Machine {
            at: 1,
            cpu: 12.0,
            memory: 40.0,
            gpu: 3.0,
            machine: machine.clone(),
        });
        state.reduce(MetricsMsg::Rates {
            at: 2,
            disk: 500,
            network: 700,
        });
        state.reduce(MetricsMsg::Machine {
            at: 3,
            cpu: 14.0,
            memory: 41.0,
            gpu: 3.0,
            machine: MachineSummary {
                cpu_percent: 14.0,
                ..machine
            },
        });

        let shown = state.machine.ready().unwrap();
        assert_eq!((shown.cpu_percent, shown.disk_bytes_per_sec, shown.network_bytes_per_sec), (14.0, 500, 700));
        assert_eq!(state.cpu_history.ready().unwrap().last(), Some((3, 14.0)));
        assert_eq!(state.disk_history.ready().unwrap().last(), Some((2, 500.0)));
    }
}
