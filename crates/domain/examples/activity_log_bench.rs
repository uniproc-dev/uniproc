use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

struct Counting;

static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);
static BYTES: AtomicUsize = AtomicUsize::new(0);
static LIVE: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        BYTES.fetch_add(layout.size(), Ordering::Relaxed);
        LIVE.fetch_add(layout.size(), Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        LIVE.fetch_sub(layout.size(), Ordering::Relaxed);
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        BYTES.fetch_add(new_size, Ordering::Relaxed);
        LIVE.fetch_add(new_size, Ordering::Relaxed);
        LIVE.fetch_sub(layout.size(), Ordering::Relaxed);
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static GLOBAL: Counting = Counting;

use app_contracts::features::activity::{Clock, Filter, Group, Hue, Pick, Span};
use app_contracts::features::agents::{ProcessCame, ProcessEvent, ProcessInstance, ProcessWent};
use domain::features::activity::log::{view, Ask, Log, Ticks};

const BASE: u64 = 1_000 * 3_600 * Ticks::Second;
const ITERATIONS: u32 = 20;

struct World {
    events: Vec<ProcessEvent>,
    next: u32,
}

impl World {
    fn spawn(&mut self, at: u64, parent: ProcessInstance, image: &str, lives: Option<u64>) -> ProcessInstance {
        self.next += 1;
        let instance = ProcessInstance {
            pid: self.next,
            sequence: u64::from(self.next),
        };
        let name = image.rsplit('\\').next().unwrap_or(image);
        self.events.push(ProcessEvent::Came(ProcessCame {
            instance,
            parent,
            at,
            image_path: image.into(),
            command_line: format!(r#""{image}" --tick {at}"#).into(),
            working_dir: r"C:\Users\someone".into(),
            user: "someone".into(),
            session_id: 1,
            elevated: Some(false),
            scheduled_task: None,
            parent_services: Arc::from([]),
        }));
        if let Some(lives) = lives {
            self.events.push(ProcessEvent::Went(ProcessWent {
                instance,
                at: at + lives,
                image_path: image.into(),
                image_name: name.into(),
                started_at: Some(at),
                ..ProcessWent::default()
            }));
        }
        instance
    }
}

fn hour(storm_every: u64) -> Vec<ProcessEvent> {
    let mut world = World {
        events: Vec::new(),
        next: 100,
    };
    let root = ProcessInstance::default();
    let services = world.spawn(BASE, root, r"C:\Windows\System32\services.exe", None);
    let schedule = world.spawn(BASE, services, r"C:\Windows\System32\svchost.exe", None);
    let explorer = world.spawn(BASE, root, r"C:\Windows\explorer.exe", None);
    let claude = world.spawn(BASE, explorer, r"C:\Users\someone\AppData\Local\claude\claude.exe", None);
    let ms = Ticks::Second / 1_000;
    let end = BASE + 3_600 * Ticks::Second;
    let mut at = BASE + Ticks::Second;
    while at < end {
        world.spawn(at, schedule, r"C:\Windows\System32\taskhostw.exe", Some(30 * ms));
        at += storm_every * ms;
    }
    let mut at = BASE + Ticks::Second;
    while at < end {
        let shell = world.spawn(at, claude, r"C:\Program Files\PowerShell\7\pwsh.exe", Some(400 * ms));
        world.spawn(at + 10 * ms, shell, r"C:\Program Files\Git\cmd\git.exe", Some(100 * ms));
        at += 2 * Ticks::Second;
    }
    let mut at = BASE + 5 * Ticks::Second;
    while at < end {
        let cargo = world.spawn(at, claude, r"C:\Users\someone\.cargo\bin\cargo.exe", Some(8 * Ticks::Second));
        for n in 0..20 {
            world.spawn(
                at + n * 200 * ms,
                cargo,
                r"C:\Users\someone\.rustup\toolchains\stable\bin\rustc.exe",
                Some(1_500 * ms),
            );
        }
        at += 60 * Ticks::Second;
    }
    world.events.sort_by_key(ProcessEvent::at);
    world.events
}

fn utc(at: u64) -> Clock {
    let seconds = at / Ticks::Second;
    Clock {
        hour: (seconds / 3_600 % 24) as u8,
        minute: (seconds / 60 % 60) as u8,
        second: (seconds % 60) as u8,
    }
}

fn groups() -> Vec<Group> {
    vec![
        Group::windows_background(),
        Group {
            id: "g1".into(),
            name: "Build".into(),
            hue: Hue::Purple,
            rules: vec![Pick::Under("cargo.exe".into())],
            shown: true,
        },
    ]
}

fn measure(label: &str, log: &Log, ask: &Ask<'_>) {
    let mut total = Duration::ZERO;
    let mut worst = Duration::ZERO;
    let mut dots = 0;
    let (mut allocations, mut bytes) = (0, 0);
    for _ in 0..ITERATIONS {
        let (counted, weighed) = (ALLOCATIONS.load(Ordering::Relaxed), BYTES.load(Ordering::Relaxed));
        let started = Instant::now();
        let built = view(log, ask);
        let took = started.elapsed();
        allocations = ALLOCATIONS.load(Ordering::Relaxed) - counted;
        bytes = BYTES.load(Ordering::Relaxed) - weighed;
        dots = built.scatter.dots.len();
        total += took;
        worst = worst.max(took);
    }
    println!(
        "{label:<22} mean {:>9.2?}  worst {:>9.2?}  {allocations:>6} allocs {:>8.1} KiB  ({dots} dots)",
        total / ITERATIONS,
        worst,
        bytes as f64 / 1024.0
    );
}

fn main() {
    let storm_every: u64 = std::env::args().nth(1).and_then(|arg| arg.parse().ok()).unwrap_or(150);
    let events = hour(storm_every);
    let comes = events.iter().filter(|event| matches!(event, ProcessEvent::Came(_))).count();
    println!("one hour, taskhostw every {storm_every} ms: {} events, {comes} starts", events.len());

    let held = LIVE.load(Ordering::Relaxed);
    let mut log = Log::default();
    let started = Instant::now();
    for batch in events.chunks(64) {
        log.record(batch);
    }
    let kept = LIVE.load(Ordering::Relaxed) - held;
    println!(
        "record: {:?} for the hour, the log holds {:.1} MiB, {} bytes a start",
        started.elapsed(),
        kept as f64 / 1024.0 / 1024.0,
        kept / comes
    );
    drop(events);

    let now = BASE + 3_600 * Ticks::Second;
    let groups = groups();
    let plain = Filter::default();
    let hiding = Filter {
        hidden: vec![Pick::Exe("git.exe".into()), Pick::Under("claude.exe".into())],
        ..Filter::default()
    };
    let searching = Filter {
        text: "rustc".into(),
        ..Filter::default()
    };
    for (name, filter) in [("plain", &plain), ("hiding", &hiding), ("search", &searching)] {
        for span in [Span::FiveMinutes, Span::Hour] {
            let ask = Ask {
                now,
                span,
                filter,
                groups: &groups,
                area: None,
                clock: utc,
            };
            measure(&format!("{name}, {span:?}"), &log, &ask);
        }
    }
}
