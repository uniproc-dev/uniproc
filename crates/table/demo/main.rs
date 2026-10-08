use std::fmt::Write;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use table::layout::Width;
use table::model::{Align, Cell, Icon, Rgba, RowKey, Source, Tone};
use table::{body, Body, Look};
use windows_reactor::{App, Callback, Component, ComponentContext, ComponentTimer, Grid, View, ViewContext};

const SVG: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32"><circle cx="16" cy="16" r="11" fill="none" stroke="#9ab4d8" stroke-width="4"/><circle cx="16" cy="16" r="4" fill="#e0a040"/></svg>"##;

const NAMES: [&str; 6] = ["svchost.exe", "chrome.exe", "Служба узла: Брандмауэр", "explorer.exe", "Code.exe", "dwm.exe"];

#[derive(Clone)]
struct Row {
    key: RowKey,
    name: String,
    values: [f32; 4],
}

struct Rows {
    rows: Vec<Row>,
    square: Arc<[u8]>,
}

fn square() -> Arc<[u8]> {
    (0..16u32 * 16)
        .flat_map(|at| {
            let (x, y) = (at % 16, at / 16);
            [(x * 16) as u8, (y * 16) as u8, 200, 255]
        })
        .collect()
}

impl Source for Rows {
    fn len(&self) -> usize {
        self.rows.len()
    }

    fn key(&self, at: usize) -> RowKey {
        self.rows[at].key
    }

    fn height(&self, _at: usize) -> f32 {
        32.0
    }

    fn columns(&self) -> usize {
        5
    }

    fn cell(&self, at: usize, column: usize, out: &mut Cell) {
        out.clear();
        let row = &self.rows[at];
        match column {
            0 => {
                out.icon = Some(if at.is_multiple_of(2) {
                    Icon::Svg(SVG)
                } else {
                    Icon::Rgba { width: 16, height: 16, pixels: self.square.clone() }
                });
                out.text.push_str(&row.name);
                if at.is_multiple_of(7) {
                    out.note.push_str("(3)");
                }
                out.dim = at.is_multiple_of(11);
            }
            1 => {
                let _ = write!(out.text, "{:.1}%", row.values[0]);
                out.align = Align::End;
                let share = (row.values[0] / 30.0).clamp(0.0, 1.0);
                out.heat = Some(Rgba { r: 0, g: 120, b: 215, a: (share.sqrt() * 230.0) as u8 });
            }
            2 => {
                let _ = write!(out.text, "{:.1} MB", row.values[1]);
                out.align = Align::End;
            }
            3 => {
                let _ = write!(out.text, "{:.1} MB/s", row.values[2]);
                out.align = Align::End;
                if row.values[2] < 1.0 {
                    out.tone = Tone::Disabled;
                }
            }
            _ => {
                let running = at.is_multiple_of(3);
                out.text.push_str(if running { "Running" } else { "Stopped" });
                if running {
                    out.tone = Tone::Success;
                }
            }
        }
    }

    fn tip(&self, at: usize, column: usize) -> Option<String> {
        let row = &self.rows[at];
        match column {
            0 => Some(format!("{}\n\nkey {}", row.name, row.key)),
            1 => Some(format!("{:.3}%", row.values[0])),
            _ => None,
        }
    }
}

struct Demo {
    rows: Rc<Rows>,
    seed: u64,
    selected: Option<RowKey>,
    _timer: Option<ComponentTimer>,
}

enum Msg {
    Tick,
    Select(Option<RowKey>),
}

impl Demo {
    fn random(&mut self) -> f32 {
        self.seed ^= self.seed << 13;
        self.seed ^= self.seed >> 7;
        self.seed ^= self.seed << 17;
        (self.seed >> 40) as f32 / (1u64 << 24) as f32
    }

    fn advance(&mut self) {
        let draws: Vec<[f32; 3]> = (0..self.rows.rows.len()).map(|_| [self.random(), self.random(), self.random()]).collect();
        let mut rows = Rows { rows: self.rows.rows.clone(), square: self.rows.square.clone() };
        for (row, r) in rows.rows.iter_mut().zip(draws) {
            row.values[0] = row.values[0] * 0.7 + r[0] * r[0] * 30.0 * 0.3;
            row.values[1] = row.values[1] * 0.95 + r[1] * 800.0 * 0.05;
            row.values[2] = r[2] * r[2] * 20.0;
        }
        if std::env::var("SORT").as_deref() != Ok("0") {
            rows.rows.sort_by(|a, b| b.values[0].total_cmp(&a.values[0]));
        }
        self.rows = Rc::new(rows);
    }
}

impl Component for Demo {
    type Input = ();
    type Message = Msg;

    fn create(_input: &(), cx: &ComponentContext<Self>) -> Self {
        let count = std::env::var("ROWS").ok().and_then(|value| value.parse().ok()).unwrap_or(2000usize);
        let rows = (0..count)
            .map(|at| Row {
                key: at as RowKey + 1,
                name: format!("{} ({at})", NAMES[at % NAMES.len()]),
                values: [0.0; 4],
            })
            .collect();
        let mut demo = Self {
            rows: Rc::new(Rows { rows, square: square() }),
            seed: 0x9e3779b97f4a7c15,
            selected: None,
            _timer: Some(cx.set_timeout(Duration::from_millis(1000), Msg::Tick)),
        };
        demo.advance();
        demo
    }

    fn update(&mut self, message: Msg, cx: &ComponentContext<Self>) {
        match message {
            Msg::Tick => {
                self.advance();
                self._timer = Some(cx.set_timeout(Duration::from_millis(1000), Msg::Tick));
            }
            Msg::Select(key) => self.selected = key,
        }
    }

    fn view(&self, _input: &(), cx: &mut ViewContext<Self>) -> View {
        let select: Callback<Option<RowKey>> = cx.callback(Msg::Select);
        Grid::new()
            .children((body(Body {
                source: self.rows.clone(),
                widths: Rc::from([Width::Fill { min: 200.0 }, Width::Fixed(84.0), Width::Fixed(96.0), Width::Fixed(96.0), Width::Fixed(90.0)]),
                look: Look {
                    hovered: Rgba { r: 60, g: 60, b: 60, a: 255 },
                    selected: Rgba { r: 0, g: 84, b: 140, a: 255 },
                    ..Look::default()
                },
                selected: self.selected,
                on_select: Some(select),
            }),))
            .into()
    }
}

fn main() -> windows_core::Result<()> {
    App::run_component::<Demo>(())
}
