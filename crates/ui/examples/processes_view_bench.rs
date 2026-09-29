use std::rc::Rc;
use std::time::{Duration, Instant};

use app_contracts::features::agents::AgentConnectionState;
use app_contracts::features::settings::ByteUnits;
use app_contracts::features::processes::{
    MachineSummary, ProcessCategory, ProcessColumn, ProcessRow, ProcessesState,
};
use guinea::prelude::{Dispatch, Load};
use guinea_plugin_l10n::Localization;
use ui::l10n::L10n;
use ui::pages::processes::{ProcessesMsg, ProcessesPage};
use ui::theme::Palette;
use windows_reactor::{Callback, ColorScheme};

const ITERATIONS: u32 = 50;

fn executables() -> Vec<String> {
    std::fs::read_dir(r"C:\Windows\System32")
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .filter(|path| path.extension().is_some_and(|ext| ext == "exe"))
                .take(140)
                .map(|path| path.to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default()
}

fn rows(exes: &[String], processes: usize, tick: u32) -> Rc<[ProcessRow]> {
    (0..processes)
        .map(|index| {
            let exe = &exes[index % exes.len()];
            let file = exe.rsplit('\\').next().unwrap_or_default();
            let name: std::sync::Arc<str> = format!("{index}-{file}").into();
            ProcessRow {
                pid: 1000 + index as u32,
                display_name: name.clone(),
                name,
                cpu_percent: ((index as u32 * 7 + tick) % 100) as f32 / 10.0,
                memory_bytes: 1_000_000 + (index as u64 * 37_111 + tick as u64 * 4_096),
                disk_bytes: (index as u64 * 13 + tick as u64) % 50_000,
                net_bytes: (index as u64 * 17 + tick as u64) % 70_000,
                gpu_percent: ((index as u32 * 3 + tick) % 50) as f32 / 10.0,
                gpu_memory_bytes: (index as u64 * 19_001) % 300_000_000,
                exe_path: exe.as_str().into(),
                package_full_name: "".into(),
                owner: None,
                owner_pid: None,
                category: ProcessCategory::ORDER[index % ProcessCategory::ORDER.len()],
                services: None,
                windows: None,
                details: Default::default(),
            }
        })
        .collect()
}

fn state(rows: Rc<[ProcessRow]>) -> ProcessesState {
    ProcessesState {
        rows: Load::Ready(rows),
        machine_summary: Load::Ready(MachineSummary {
            cpu_percent: 12.5,
            cpu_current_mhz: 3_200,
            cpu_max_mhz: 4_800,
            memory_used_bytes: 12_000_000_000,
            memory_total_bytes: 32_000_000_000,
            gpu_percent: 7.5,
            gpu_memory_used_bytes: 1_500_000_000,
            ..MachineSummary::default()
        }),
        selected: Some(1_010),
        sort_column: ProcessColumn::Cpu,
        descending: true,
        agent_state: AgentConnectionState::Connected,
        ..ProcessesState::default()
    }
}

fn main() {
    let processes: usize = std::env::args()
        .nth(1)
        .and_then(|arg| arg.parse().ok())
        .unwrap_or(400);
    let exes = executables();
    assert!(!exes.is_empty(), "no executables found to take icons from");

    let l10n = L10n::for_tag("en").expect("english strings");
    let palette = Palette::of(ColorScheme::Dark);
    let dispatch = Dispatch::default();
    let forward = Callback::new(|_: ProcessesMsg| {});
    let open_settings = Callback::new(|()| {});
    let page = ProcessesPage::new(None);

    let first = state(rows(&exes, processes, 0));
    let started = Instant::now();
    drop(page.view(&first, &dispatch, &l10n, palette, forward.clone(), open_settings.clone(), ByteUnits::Windows));
    println!("first view (cold icon cache): {:?}", started.elapsed());
    std::thread::sleep(Duration::from_secs(3));

    let mut total = Duration::ZERO;
    let mut worst = Duration::ZERO;
    for tick in 1..=ITERATIONS {
        let next = state(rows(&exes, processes, tick));
        let started = Instant::now();
        let view = page.view(&next, &dispatch, &l10n, palette, forward.clone(), open_settings.clone(), ByteUnits::Windows);
        let took = started.elapsed();
        drop(view);
        total += took;
        worst = worst.max(took);
    }
    println!(
        "warm view, {processes} distinct rows: mean {:?}, worst {:?} over {ITERATIONS} ticks",
        total / ITERATIONS,
        worst
    );
}
