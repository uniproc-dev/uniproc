use std::collections::HashMap;

use tokio::time::Instant;

use app_contracts::features::processes::ProcessRow;

#[derive(Debug, Default)]
pub struct IoRates {
    at: Option<Instant>,
    previous: HashMap<u32, (u64, u64)>,
}

fn per_second(now: u64, before: u64, seconds: f64) -> u64 {
    now.checked_sub(before)
        .map_or(0, |delta| (delta as f64 / seconds).round() as u64)
}

impl IoRates {
    #[tracing::instrument(skip_all, level = "debug", fields(rows = rows.len()))]
    pub fn apply(&mut self, rows: &mut [ProcessRow], now: Instant) {
        let seconds = self
            .at
            .map(|at| now.duration_since(at).as_secs_f64())
            .filter(|seconds| *seconds > 0.0);
        let mut current = HashMap::with_capacity(rows.len());

        for row in rows.iter_mut() {
            let counters = (row.disk_bytes, row.net_bytes);
            let (disk, net) = match (seconds, self.previous.get(&row.pid)) {
                (Some(seconds), Some(&(disk, net))) => (
                    per_second(counters.0, disk, seconds),
                    per_second(counters.1, net, seconds),
                ),
                _ => (0, 0),
            };
            current.insert(row.pid, counters);
            row.disk_bytes = disk;
            row.net_bytes = net;
        }

        self.previous = current;
        self.at = Some(now);
    }

    #[cfg(test)]
    fn tracked(&self) -> usize {
        self.previous.len()
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
