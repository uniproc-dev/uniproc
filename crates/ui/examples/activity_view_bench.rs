use std::rc::Rc;
use std::sync::Arc;
use std::time::{Duration, Instant};

use app_contracts::features::activity::{
    ActivityRow, ActivityState, ActivityView, Came, Clock, Dot, Exit, Launcher, Lived, Pick, Scatter, Series, Went,
};
use app_contracts::features::agents::ProcessInstance;
use guinea::prelude::{Dispatch, Load};
use guinea_plugin_l10n::Localization;
use ui::l10n::L10n;
use ui::pages::activity::{ActivityPage, ActivityPageMsg};
use ui::theme::Palette;
use windows_reactor::{Callback, ColorScheme};

const ITERATIONS: u32 = 50;
const ROWS: usize = 200;

fn instance(index: usize) -> ProcessInstance {
    ProcessInstance {
        pid: 1000 + index as u32,
        sequence: index as u64,
    }
}

fn clock(index: usize) -> Clock {
    Clock {
        hour: 12,
        minute: (index / 60 % 60) as u8,
        second: (index % 60) as u8,
    }
}

fn exit(index: usize) -> Exit {
    Exit {
        at: clock(index + 3),
        code: 0,
        lived: 1_000 + index as u64 * 37,
        cpu_cycles: 0,
        read_bytes: 0,
        write_bytes: 0,
        peak_commit_bytes: 0,
    }
}

fn came(index: usize) -> Came {
    let name: Arc<str> = format!("tool-{}.exe", index % 40).into();
    Came {
        key: instance(index),
        at: clock(index),
        name: name.clone(),
        image_path: format!(r"C:\Program Files\Tool {}\{name}", index % 7).into(),
        command_line: format!(r#""{name}" --run {index}"#).into(),
        working_dir: r"C:\Users\someone".into(),
        user: "someone".into(),
        session_id: 1,
        elevated: Some(false),
        launcher: match index % 3 {
            0 => Launcher::Process("explorer.exe".into()),
            1 => Launcher::Services(Arc::from([Arc::<str>::from("Schedule")])),
            _ => Launcher::Unknown,
        },
        chain: vec!["explorer.exe".into()],
        parent_services: Arc::from([]),
        first_seen: index.is_multiple_of(9),
        exit: index.is_multiple_of(2).then(|| exit(index)),
        picks: vec![Pick::Exe(name), Pick::Folder(r"c:\program files\tool".into())],
    }
}

fn row(index: usize) -> ActivityRow {
    match index % 10 {
        0 => ActivityRow::Series(Rc::new(Series {
            key: instance(index),
            at: clock(index),
            launcher: "cargo.exe".into(),
            folder: r"c:\target\debug".into(),
            names: vec![("rustc.exe".into(), 4), ("build-script.exe".into(), 2)],
            count: 6,
            went: 5,
            routine: false,
            members: (0..6).map(|member| Rc::new(came(index * 10 + member))).collect(),
            picks: vec![Pick::Launcher("cargo.exe".into())],
        })),
        1 | 2 => ActivityRow::Went(Rc::new(Went {
            key: instance(index),
            name: Some(format!("gone-{index}.exe").into()),
            lived: Some(5_000),
            exit: exit(index),
        })),
        _ => ActivityRow::Came(Rc::new(came(index))),
    }
}

fn state(dots: usize, rows: usize) -> ActivityState {
    let now = 3_600_000;
    let scatter = Scatter {
        dots: (0..dots)
            .map(|index| Dot {
                key: instance(index),
                at: now - (index as u64 * 1_237) % now,
                lived: match index % 5 {
                    0 => Lived::Running,
                    1 => Lived::Unknown,
                    _ => Lived::For(1 + (index as u64 * 7_919) % 36_000),
                },
                faint: index % 4 == 0,
            })
            .collect(),
        now,
        now_clock: clock(0),
        length: now,
        area: None,
    };
    ActivityState {
        view: Load::Ready(Rc::new(ActivityView {
            scatter: Rc::new(scatter),
            rows: (0..rows).map(row).collect(),
            earlier: 40,
            came: rows,
            went: rows / 5,
            from: clock(0),
            to: clock(3_599),
            history_since: None,
            lost: 0,
        })),
        ..ActivityState::default()
    }
}

fn measure(label: &str, page: &ActivityPage, state: &ActivityState, l10n: &L10n, palette: Palette) {
    let dispatch = Dispatch::default();
    let forward = Callback::new(|_: ActivityPageMsg| {});
    drop(page.view(state, &dispatch, l10n, palette, forward.clone()));

    let mut total = Duration::ZERO;
    let mut worst = Duration::ZERO;
    for _ in 0..ITERATIONS {
        let started = Instant::now();
        let view = page.view(state, &dispatch, l10n, palette, forward.clone());
        let took = started.elapsed();
        drop(view);
        total += took;
        worst = worst.max(took);
    }
    println!("{label}: mean {:?}, worst {worst:?} over {ITERATIONS} renders", total / ITERATIONS);
}

fn main() {
    let dots: usize = std::env::args()
        .nth(1)
        .and_then(|arg| arg.parse().ok())
        .unwrap_or(3_000);
    let l10n = L10n::for_tag("en").expect("english strings");
    let palette = Palette::of(ColorScheme::Dark);
    let page = ActivityPage::default();

    measure("empty", &page, &state(0, 0), &l10n, palette);
    measure(&format!("{dots} dots, no rows"), &page, &state(dots, 0), &l10n, palette);
    measure(&format!("no dots, {ROWS} rows"), &page, &state(0, ROWS), &l10n, palette);
    measure(&format!("{dots} dots, {ROWS} rows"), &page, &state(dots, ROWS), &l10n, palette);
}
