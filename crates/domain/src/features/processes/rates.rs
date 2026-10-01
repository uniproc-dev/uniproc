use std::collections::{HashMap, VecDeque};
use std::time::Duration;

use tokio::time::Instant;

use app_contracts::features::processes::ProcessRow;

type Counters = HashMap<u32, (u64, u64)>;

struct Window;

#[expect(non_upper_case_globals)]
impl Window {
    const Span: Duration = Duration::from_secs(1);
}

#[derive(Debug, Default)]
pub struct IoRates {
    seen: VecDeque<(Instant, Counters)>,
}

fn per_second(now: u64, before: u64, seconds: f64) -> u64 {
    now.checked_sub(before)
        .map_or(0, |delta| (delta as f64 / seconds).round() as u64)
}

impl IoRates {
    #[tracing::instrument(skip_all, level = "debug", fields(rows = rows.len()))]
    pub fn apply(&mut self, rows: &mut [ProcessRow], now: Instant) {
        if let Some(start) = now.checked_sub(Window::Span) {
            while self.seen.get(1).is_some_and(|(at, _)| *at <= start) {
                self.seen.pop_front();
            }
        }
        let current: Counters = rows
            .iter()
            .map(|row| (row.pid, (row.disk_bytes, row.net_bytes)))
            .collect();

        for row in rows.iter_mut() {
            let since = self.seen.iter().find_map(|(at, counters)| {
                let seconds = now.duration_since(*at).as_secs_f64();
                counters.get(&row.pid).filter(|_| seconds > 0.0).map(|&before| (before, seconds))
            });
            let (disk, net) = match since {
                Some(((disk, net), seconds)) => (
                    per_second(row.disk_bytes, disk, seconds),
                    per_second(row.net_bytes, net, seconds),
                ),
                None => (0, 0),
            };
            row.disk_bytes = disk;
            row.net_bytes = net;
        }

        for (_, counters) in &mut self.seen {
            counters.retain(|pid, _| current.contains_key(pid));
        }
        self.seen.push_back((now, current));
    }

    #[cfg(test)]
    fn tracked(&self) -> usize {
        self.seen
            .iter()
            .flat_map(|(_, counters)| counters.keys())
            .collect::<std::collections::HashSet<_>>()
            .len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn row(pid: u32, disk: u64, net: u64) -> ProcessRow {
        ProcessRow {
            pid,
            name: "p.exe".into(),
            display_name: "p".into(),
            cpu_percent: 0.0,
            memory_bytes: 0,
            disk_bytes: disk,
            net_bytes: net,
            gpu_percent: 0.0,
            gpu_memory_bytes: 0,
            exe_path: "".into(),
            package_full_name: "".into(),
            owner: None,
            owner_pid: None,
            category: app_contracts::features::processes::ProcessCategory::App,
            services: None,
            windows: None,
            details: Default::default(),
            is_monitor: false,
        }
    }

    #[test]
    fn a_counter_becomes_bytes_per_second() {
        let mut rates = IoRates::default();
        let start = Instant::now();
        rates.apply(&mut [row(1, 1_000, 500)], start);

        let mut rows = [row(1, 3_000, 1_500)];
        rates.apply(&mut rows, start + Duration::from_millis(500));

        assert_eq!(rows[0].disk_bytes, 4_000);
        assert_eq!(rows[0].net_bytes, 2_000);
    }

    #[test]
    fn a_burst_in_one_short_tick_counts_over_the_whole_second() {
        let mut rates = IoRates::default();
        let start = Instant::now();
        let tick = |ms: u64, disk: u64, rates: &mut IoRates| {
            let mut rows = [row(1, disk, 0)];
            rates.apply(&mut rows, start + Duration::from_millis(ms));
            rows[0].disk_bytes
        };
        for ms in [0, 200, 400, 600, 800] {
            tick(ms, 0, &mut rates);
        }
        assert_eq!(tick(1_000, 1_000, &mut rates), 1_000, "not 5000/s from 1000 bytes in 200 ms");
        assert_eq!(tick(1_200, 1_000, &mut rates), 1_000, "still inside the last second");
        assert_eq!(tick(2_000, 1_000, &mut rates), 0, "the burst left the window");
    }

    #[test]
    fn the_first_sight_of_a_process_has_no_rate() {
        let mut rates = IoRates::default();
        let start = Instant::now();
        rates.apply(&mut [row(1, 10, 10)], start);

        let mut rows = [row(1, 20, 20), row(2, 9_999, 9_999)];
        rates.apply(&mut rows, start + Duration::from_secs(1));

        assert_eq!((rows[1].disk_bytes, rows[1].net_bytes), (0, 0));
    }

    #[test]
    fn a_counter_that_went_down_is_a_reset_not_a_negative_rate() {
        let mut rates = IoRates::default();
        let start = Instant::now();
        rates.apply(&mut [row(1, 5_000, 5_000)], start);

        let mut rows = [row(1, 100, 100)];
        rates.apply(&mut rows, start + Duration::from_secs(1));

        assert_eq!((rows[0].disk_bytes, rows[0].net_bytes), (0, 0));
    }

    #[test]
    fn exited_processes_are_forgotten() {
        let mut rates = IoRates::default();
        let start = Instant::now();
        rates.apply(&mut [row(1, 0, 0), row(2, 0, 0), row(3, 0, 0)], start);
        rates.apply(&mut [row(2, 0, 0)], start + Duration::from_secs(1));

        assert_eq!(rates.tracked(), 1);
    }
}
