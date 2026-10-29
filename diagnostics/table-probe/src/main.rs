use std::cell::{Cell as Flag, RefCell};
use std::rc::Rc;
use std::time::{Duration, Instant};
use windows_canvas::{
    CanvasImageSource, ColorF, GpuDevice, ParagraphAlignment, Rect, TextFormat, WordWrapping,
};
use windows_core::Interface;
use windows_reactor::*;

#[allow(
    clippy::all,
    dead_code,
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals
)]
mod c {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

#[derive(Default)]
struct Stats {
    creates: u64,
    drops: u64,
    inputs: u64,
    views: u64,
    draws: u64,
    draw_us: u128,
    builds: u64,
    attached: u64,
    attach_errors: u64,
    active_attach_errors: u64,
}
thread_local! {
    static STATS: RefCell<Stats> = RefCell::new(Stats::default());
    static ROOT: RefCell<Option<c::UIElement>> = const { RefCell::new(None) };
    static OFFSET: RefCell<Option<f64>> = const { RefCell::new(None) };
}

#[derive(Clone, PartialEq)]
struct Data {
    index: usize,
    values: [u64; 4],
    mode: String,
}

struct Row {
    live: Rc<Flag<bool>>,
    surface: Option<CanvasImageSource>,
    image: ElementRef<Image>,
    format: Option<TextFormat>,
}

thread_local! { static GPU: GpuDevice = GpuDevice::new().unwrap(); }

impl Row {
    fn draw(&self, data: &Data) {
        if let Some(surface) = &self.surface {
            let started = Instant::now();
            assert!(
                surface
                    .draw(ColorF::TRANSPARENT, |session| {
                        let brush = session.create_solid_brush(ColorF::BLACK)?;
                        for (i, value) in data.values.iter().enumerate() {
                            session.draw_text(
                                &format!("{value:>10}"),
                                self.format.as_ref().unwrap(),
                                &Rect::from_xywh(i as f32 * 120.0, 0.0, 120.0, 32.0),
                                &brush,
                            );
                        }
                        Ok(())
                    })
                    .unwrap()
            );
            STATS.with_borrow_mut(|s| {
                s.draws += 1;
                s.draw_us += started.elapsed().as_micros();
            });
        }
    }
}

impl Drop for Row {
    fn drop(&mut self) {
        self.live.set(false);
        STATS.with_borrow_mut(|s| s.drops += 1);
    }
}

impl Component for Row {
    type Input = Data;
    type Message = ();
    fn create(data: &Data, _: &ComponentContext<Self>) -> Self {
        STATS.with_borrow_mut(|s| s.creates += 1);
        let row = Self {
            live: Rc::new(Flag::new(true)),
            surface: (data.mode == "dwrite").then(|| {
                GPU.with(|device| CanvasImageSource::new(device, 480.0, 32.0, 1.0).unwrap())
            }),
            image: ElementRef::new(),
            format: (data.mode == "dwrite").then(|| {
                TextFormat::new("Consolas", 14.0)
                    .unwrap()
                    .with_word_wrapping(WordWrapping::NoWrap)
                    .with_paragraph_alignment(ParagraphAlignment::Center)
            }),
        };
        row.draw(data);
        row
    }
    fn input_changed(&mut self, data: &Data, _: &ComponentContext<Self>) {
        STATS.with_borrow_mut(|s| s.inputs += 1);
        self.draw(data);
    }
    fn update(&mut self, _: (), _: &ComponentContext<Self>) {}
    fn view(&self, data: &Data, cx: &mut ViewContext<Self>) -> View {
        STATS.with_borrow_mut(|s| s.views += 1);
        let metrics: View = if let Some(surface) = &self.surface {
            let surface = surface.clone();
            let image = self.image.clone();
            let live = self.live.clone();
            cx.use_effect("surface", (), move || {
                let queued = surface.attach_result(&image, move |result| {
                    STATS.with_borrow_mut(|s| {
                        if result.is_ok() {
                            s.attached += 1;
                        } else {
                            s.attach_errors += 1;
                            if live.get() {
                                s.active_attach_errors += 1;
                            }
                        }
                    })
                });
                if !queued {
                    STATS.with_borrow_mut(|s| s.attach_errors += 1);
                }
                None
            });
            Image::new()
                .element_ref(&self.image)
                .width(480.0)
                .height(32.0)
                .into()
        } else if data.mode == "joined" {
            TextBlock::new()
                .font_family("Consolas")
                .font_size(14.0)
                .text(
                    data.values
                        .iter()
                        .map(|v| format!("{v:>10}"))
                        .collect::<Vec<_>>()
                        .join("    "),
                )
                .foreground(Color::rgb(0, 0, 0))
                .width(480.0)
                .into()
        } else if data.mode == "positioned" {
            Canvas::new()
                .width(480.0)
                .height(32.0)
                .children(
                    data.values
                        .iter()
                        .enumerate()
                        .map(|(i, v)| {
                            TextBlock::new()
                                .canvas_left(i as f64 * 120.0)
                                .canvas_top(0.0)
                                .width(120.0)
                                .font_family("Consolas")
                                .font_size(14.0)
                                .text(format!("{v:>10}"))
                                .foreground(Color::rgb(0, 0, 0))
                                .into()
                        })
                        .collect::<Vec<View>>(),
                )
                .into()
        } else {
            Grid::new()
                .columns([GridLength::Star(1.0); 4])
                .children(
                    data.values
                        .iter()
                        .enumerate()
                        .map(|(i, v)| {
                            TextBlock::new()
                                .grid_column(i as i32)
                                .font_family("Consolas")
                                .font_size(14.0)
                                .text(format!("{v:>10}"))
                                .foreground(Color::rgb(0, 0, 0))
                                .into()
                        })
                        .collect::<Vec<View>>(),
                )
                .into()
        };
        Border::new()
            .height(32.0)
            .background(if data.index.is_multiple_of(2) {
                Color::rgb(240, 240, 240)
            } else {
                Color::rgb(255, 255, 255)
            })
            .content(
                Grid::new()
                    .columns([GridLength::Pixel(240.0), GridLength::Pixel(480.0)])
                    .children((
                        TextBlock::new()
                            .text(format!("Process {:05}", data.index))
                            .font_size(14.0)
                            .foreground(Color::rgb(0, 0, 0)),
                        Border::new().grid_column(1).content(metrics),
                    )),
            )
            .into()
    }
}

fn cpu() -> u64 {
    unsafe {
        let mut created = Default::default();
        let mut exited = Default::default();
        let mut kernel = c::FILETIME::default();
        let mut user = c::FILETIME::default();
        c::GetProcessTimes(
            c::GetCurrentProcess(),
            &mut created,
            &mut exited,
            &mut kernel,
            &mut user,
        )
        .unwrap();
        ((kernel.dwHighDateTime as u64) << 32 | kernel.dwLowDateTime as u64)
            + ((user.dwHighDateTime as u64) << 32 | user.dwLowDateTime as u64)
    }
}

fn memory() -> usize {
    unsafe {
        let mut info = c::PROCESS_MEMORY_COUNTERS_EX::default();
        c::GetProcessMemoryInfo(
            c::GetCurrentProcess(),
            (&mut info as *mut c::PROCESS_MEMORY_COUNTERS_EX).cast(),
            std::mem::size_of_val(&info) as u32,
        )
        .unwrap();
        info.PrivateUsage
    }
}

struct Probe {
    _focus: windows_core::EventRevoker,
    focus: ElementRef<Button>,
    timer: Option<ComponentTimer>,
    rows: Rc<Vec<[u64; 4]>>,
    order: Rc<Vec<usize>>,
    mixing: String,
    moved: u64,
    displacement: u64,
    mode: String,
    period: u64,
    start: Instant,
    last: Instant,
    next: Instant,
    tick: u64,
    phase: u32,
    phase_start: Instant,
    cpu_start: u64,
    phase_tick: u64,
    delays: Vec<f64>,
    model_us: u128,
    source: RefCell<Option<(u64, VirtualSource)>>,
}

impl Probe {
    fn report(&self) {
        let mut delays = self.delays.clone();
        delays.sort_by(f64::total_cmp);
        STATS.with_borrow(|s| eprintln!("RESULT {}", serde_json::json!({
            "mode":self.mode,"rows":self.rows.len(),"period_ms":self.period,"phase":self.phase,
            "ticks":self.tick-self.phase_tick,
            "mixing":self.mixing,"moved_positions":self.moved,"rank_distance":self.displacement,
            "elapsed_s":self.phase_start.elapsed().as_secs_f64(),"cpu_ms":(cpu()-self.cpu_start) as f64/10000.0,
            "private_mib":memory() as f64/1048576.0,"creates":s.creates,"drops":s.drops,"inputs":s.inputs,"views":s.views,
            "draws":s.draws,"draw_us":s.draw_us,"builds":s.builds,"model_us":self.model_us,
            "attached":s.attached,"attach_errors":s.attach_errors,
            "active_attach_errors":s.active_attach_errors,
            "heartbeat_p95_ms":delays.get(delays.len()*95/100),"heartbeat_max_ms":delays.last()
            ,"heartbeat_p99_ms":delays.get(delays.len()*99/100)
            ,"scroll_offset":OFFSET.with_borrow(|offset| *offset)
        })));
    }
}

impl Component for Probe {
    type Input = ();
    type Message = ();
    fn create(_: &(), cx: &ComponentContext<Self>) -> Self {
        let focus = c::FocusManager::GotFocus(|_, args| {
            let root = (|| -> windows_core::Result<c::UIElement> {
                args.ok()?
                    .NewFocusedElement()?
                    .cast::<c::UIElement>()?
                    .XamlRoot()?
                    .Content()
            })();
            if let Ok(root) = root {
                ROOT.with_borrow_mut(|slot| *slot = Some(root));
            }
        })
        .unwrap();
        let now = Instant::now();
        let count = std::env::var("ROWS")
            .unwrap_or_else(|_| "2000".into())
            .parse()
            .unwrap();
        Self {
            _focus: focus,
            focus: ElementRef::new(),
            timer: Some(cx.set_timeout(Duration::from_millis(20), ())),
            rows: Rc::new(vec![[0; 4]; count]),
            order: Rc::new((0..count).collect()),
            mixing: std::env::var("MIX").unwrap_or_else(|_| "mild".into()),
            moved: 0,
            displacement: 0,
            mode: std::env::var("MODE").unwrap_or_else(|_| "native".into()),
            period: std::env::var("PERIOD")
                .unwrap_or_else(|_| "500".into())
                .parse()
                .unwrap(),
            start: now,
            last: now,
            next: now,
            tick: 0,
            phase: 0,
            phase_start: now,
            cpu_start: cpu(),
            phase_tick: 0,
            delays: Vec::new(),
            model_us: 0,
            source: RefCell::new(None),
        }
    }
    fn update(&mut self, _: (), cx: &ComponentContext<Self>) {
        let now = Instant::now();
        self.delays
            .push(now.duration_since(self.last).as_secs_f64() * 1000.0);
        self.last = now;
        let elapsed = self.start.elapsed().as_secs_f64();
        let phase = if elapsed < 3.0 {
            0
        } else if elapsed < 9.0 {
            1
        } else if elapsed < 19.0 {
            2
        } else if elapsed < 29.0 {
            3
        } else {
            4
        };
        if phase != self.phase {
            self.report();
            self.phase = phase;
            self.phase_start = now;
            self.cpu_start = cpu();
            self.phase_tick = self.tick;
            self.delays.clear();
            self.model_us = 0;
            self.moved = 0;
            self.displacement = 0;
            STATS.with_borrow_mut(|s| *s = Stats::default());
        }
        if phase == 4 {
            self.timer = None;
            let _ = cx.close_window();
            return;
        }
        if phase != 1 && now >= self.next {
            let started = Instant::now();
            self.tick += 1;
            self.rows = Rc::new(
                (0..self.rows.len())
                    .map(|i| {
                        std::array::from_fn(|j| {
                            let mut x = (i as u64).wrapping_mul(0x9e3779b97f4a7c15)
                                ^ self.tick.wrapping_mul(0xbf58476d1ce4e5b9)
                                ^ j as u64;
                            x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
                            x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
                            let random = (x ^ (x >> 31)) % 99999999;
                            if j == 0 && self.mixing == "mild" {
                                ((i as u64 * 7919) % 99999999) + random % 2000000
                            } else {
                                random
                            }
                        })
                    })
                    .collect(),
            );
            if phase == 0 || phase == 3 {
                let mut order = (*self.order).clone();
                order.sort_unstable_by_key(|&id| (std::cmp::Reverse(self.rows[id][0]), id));
                let mut previous = vec![0usize; order.len()];
                for (at, &id) in self.order.iter().enumerate() {
                    previous[id] = at;
                }
                for (at, &id) in order.iter().enumerate() {
                    self.moved += u64::from(previous[id] != at);
                    self.displacement += previous[id].abs_diff(at) as u64;
                }
                self.order = Rc::new(order);
            }
            self.model_us += started.elapsed().as_micros();
            self.next += Duration::from_millis(self.period);
            if self.next < now {
                self.next = now + Duration::from_millis(self.period);
            }
        }
        self.timer = Some(cx.set_timeout(Duration::from_millis(20), ()));
    }
    fn view(&self, _: &(), cx: &mut ViewContext<Self>) -> View {
        let focus = self.focus.clone();
        cx.use_effect("focus", (), move || {
            let _ = focus.request_focus();
            None
        });
        let source = {
            let mut cached = self.source.borrow_mut();
            if cached.as_ref().is_none_or(|(tick, _)| *tick != self.tick) {
                let rows = self.rows.clone();
                let order = self.order.clone();
                let keys = order.clone();
                let mode = self.mode.clone();
                *cached = Some((
                    self.tick,
                    VirtualSource::new(
                        self.tick,
                        rows.len(),
                        move |i| keys[i],
                        move |i| {
                            STATS.with_borrow_mut(|s| s.builds += 1);
                            View::component::<Row>(Data {
                                index: order[i],
                                values: rows[order[i]],
                                mode: mode.clone(),
                            })
                        },
                    ),
                ));
            }
            cached.as_ref().unwrap().1.clone()
        };
        cx.window_frame(
            format!(
                "Table probe {} {} rows {}ms",
                self.mode,
                self.rows.len(),
                self.period
            ),
            Grid::new()
                .width(740.0)
                .height(560.0)
                .rows([GridLength::Auto, GridLength::Star(1.0)])
                .children((
                    Button::new()
                        .element_ref(&self.focus)
                        .content("Virtual table benchmark"),
                    ScrollViewer::new()
                        .grid_row(1)
                        .horizontal_scroll_bar_visibility(ScrollBarVisibility::Disabled)
                        .content(ItemsRepeater::new().virtual_source(source)),
                )),
        )
    }
}

fn main() {
    App::run_component::<Probe>(()).unwrap();
}
