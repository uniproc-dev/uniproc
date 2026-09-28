use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

struct Counting;

static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);
static BYTES: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        BYTES.fetch_add(layout.size(), Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        BYTES.fetch_add(new_size, Ordering::Relaxed);
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static GLOBAL: Counting = Counting;

#[compio::main]
async fn main() -> anyhow::Result<()> {
    bench::run().await
}

mod bench {
    use super::{ALLOCATIONS, BYTES};
    use anyhow::Context;
    use app_contracts::features::processes::ProcessRow;
    use domain::features::agents::windows_report::{self, Reports};
    use domain::features::processes::{rows_from_report, windows_scan};
    use uniproc_windows_agent::agent::Agent;
    use std::rc::Rc;
    use std::sync::atomic::Ordering;
    use std::time::{Duration, Instant};

    const ROUNDS: usize = 200;

    struct Sample {
        median: Duration,
        worst: Duration,
        allocations: usize,
        bytes: usize,
    }

    fn counters() -> (usize, usize) {
        (ALLOCATIONS.load(Ordering::Relaxed), BYTES.load(Ordering::Relaxed))
    }

    fn measure<I, T>(mut setup: impl FnMut() -> I, mut step: impl FnMut(I) -> T) -> Sample {
        drop(step(setup()));

        let mut times = Vec::with_capacity(ROUNDS);
        let mut allocations = 0;
        let mut bytes = 0;
        for _ in 0..ROUNDS {
            let input = setup();
            let (count_before, bytes_before) = counters();
            let started = Instant::now();
            let output = step(input);
            let took = started.elapsed();
            let (count_after, bytes_after) = counters();
            drop(output);
            times.push(took);
            allocations += count_after - count_before;
            bytes += bytes_after - bytes_before;
        }

        times.sort();
        Sample {
            median: times[ROUNDS / 2],
            worst: times[ROUNDS - 1],
            allocations: allocations / ROUNDS,
            bytes: bytes / ROUNDS,
        }
    }

    fn print(label: &str, sample: &Sample) {
        println!(
            "{label:<28} median {:>8.1} us   worst {:>8.1} us   allocations {:>6}   bytes {:>9}",
            sample.median.as_secs_f64() * 1e6,
            sample.worst.as_secs_f64() * 1e6,
            sample.allocations,
            sample.bytes,
        );
    }

    pub async fn run() -> anyhow::Result<()> {
        let agent = Agent::remote(Duration::from_secs(5))
            .await
            .context("connect failed - is uniproc-windows-agent running?")?;
        let mut watch = agent.watch(windows_report::spec(Duration::from_secs(1))).await?;
        let update = watch.next().await?;
        let snapshot = &update.snapshot;

        let mut reports = Reports::default();
        let report = reports.report(&update);
        let windowed = windows_scan::app_windows();
        println!(
            "live: {} passports, {} metric rows, {} app-window pids, {} rounds per stage\n",
            snapshot.processes.value.len(),
            report.processes.len(),
            windowed.len(),
            ROUNDS,
        );

        let joined = measure(|| (), |()| reports.report(&update));
        let scanned = measure(|| (), |()| windows_scan::app_windows());
        let mapped = measure(|| (), |()| rows_from_report(&report, &windowed));
        let published = measure(
            || rows_from_report(&report, &windowed),
            |rows| Rc::<[ProcessRow]>::from(rows),
        );

        print("join metrics to passports", &joined);
        print("app_windows", &scanned);
        print("rows_from_report", &mapped);
        print("Rc<[ProcessRow]>::from", &published);

        let median: Duration = [&joined, &scanned, &mapped, &published]
            .iter()
            .map(|s| s.median)
            .sum();
        let allocations: usize = [&joined, &scanned, &mapped, &published]
            .iter()
            .map(|s| s.allocations)
            .sum();
        println!(
            "\nprocesses path per tick     median {:>8.1} us                          allocations {:>6}",
            median.as_secs_f64() * 1e6,
            allocations,
        );

        Ok(())
    }
}
