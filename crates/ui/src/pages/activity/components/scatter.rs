use std::rc::Rc;
use std::time::{Duration, Instant};

use app_contracts::features::activity::{Area, Dot, Scatter};
use app_contracts::features::agents::ProcessInstance;
use guinea::winui::MarkExt;
use windows_reactor::{
    Border, Callback, Canvas, CanvasChildExt, ChildrenControl, Color, Component, ComponentContext, ComponentTimer,
    CompositionHostEvent, ContentControl, ElementObservation, ElementRef, Ellipse, Grid, LayoutControl,
    PointerEventInfo, Rectangle, Thickness, VerticalAlignment, View, ViewContext,
};

use super::super::marks::ActivityMark;
use super::timeline::{ticks, Scale};
use crate::format;
use crate::theme::{accent_color, space, Palette};
use crate::widgets::text::caption;

pub struct Plot;

#[expect(non_upper_case_globals)]
impl Plot {
    pub const Height: f64 = 200.0;
    const Axis: f64 = 18.0;
    const Assumed: f64 = 800.0;
    const Top: f64 = 8.0;
    const Bottom: f64 = 6.0;
    const Lowest: f32 = -0.14;
    const Highest: f32 = 1.06;
    const Radius: f64 = 2.6;
    const Ring: f64 = 4.5;
    const RingStroke: f64 = 1.2;
    const Tick: f64 = 1.4;
    const Reach: f64 = 8.0;
    const Click: f64 = 4.0;
    const Card: f64 = 320.0;
    const Offset: f64 = 14.0;
    const Labels: f64 = 48.0;
    const AreaAlpha: u8 = 40;
    const Frame: Duration = Duration::from_millis(250);
}

fn y_px(y: f32) -> f64 {
    let share = f64::from(Plot::Highest - y) / f64::from(Plot::Highest - Plot::Lowest);
    Plot::Top + share * (Plot::Height - Plot::Top - Plot::Bottom)
}

fn y_of(px: f64) -> f32 {
    let share = ((px - Plot::Top) / (Plot::Height - Plot::Top - Plot::Bottom)) as f32;
    Plot::Highest - share * (Plot::Highest - Plot::Lowest)
}

pub fn nearest(scatter: &Scatter, scale: &Scale, at: (f64, f64)) -> Option<(ProcessInstance, f64, f64)> {
    scatter
        .dots
        .iter()
        .chain(&scatter.orphans)
        .map(|dot| {
            let (x, y) = (scale.x(dot.at), y_px(dot.y));
            (dot.key, x, y, (x - at.0).hypot(y - at.1))
        })
        .filter(|(.., distance)| *distance <= Plot::Reach)
        .min_by(|a, b| a.3.total_cmp(&b.3))
        .map(|(key, x, y, _)| (key, x, y))
}

fn faint(color: Color) -> Color {
    Color { a: Plot::AreaAlpha, ..color }
}

fn marked(left: f64, right: f64, top: f64, bottom: f64, accent: Color) -> View {
    Rectangle::new()
        .canvas_left(left.min(right))
        .canvas_top(top.min(bottom))
        .width((right - left).abs())
        .height((bottom - top).abs())
        .fill(faint(accent))
        .stroke(accent)
        .stroke_thickness(1.0)
        .into()
}

#[derive(Clone, PartialEq)]
struct Layer {
    scatter: Rc<Scatter>,
    origin: u64,
    per_tick: f64,
    palette: Palette,
}

impl Layer {
    fn x(&self, at: u64) -> f64 {
        (at as f64 - self.origin as f64) * self.per_tick
    }
}

struct Dots;

impl Component for Dots {
    type Input = Layer;
    type Message = ();

    fn create(_layer: &Layer, _cx: &ComponentContext<Self>) -> Self {
        Self
    }

    fn view(&self, layer: &Layer, _cx: &mut ViewContext<Self>) -> View {
        let palette = layer.palette;
        let scatter = &layer.scatter;
        let mut items: Vec<(String, View)> = Vec::with_capacity(scatter.dots.len() + scatter.orphans.len() + 8);
        if let Some(area) = &scatter.area {
            items.push((
                "area".into(),
                marked(layer.x(area.from), layer.x(area.to), y_px(area.top), y_px(area.bottom), accent_color()),
            ));
        }
        for dot in &scatter.dots {
            let (x, y) = (layer.x(dot.at), y_px(dot.y));
            let shape = Ellipse::new()
                .canvas_left(x - Plot::Radius)
                .canvas_top(y - Plot::Radius)
                .width(Plot::Radius * 2.0)
                .height(Plot::Radius * 2.0);
            let color = if dot.faint { faint(palette.success) } else { palette.success };
            let shape = if dot.alive {
                shape.stroke(color).stroke_thickness(Plot::RingStroke)
            } else {
                shape.fill(color)
            };
            items.push((key(dot), shape.into()));
        }
        for dot in &scatter.orphans {
            items.push((
                key(dot),
                Rectangle::new()
                    .canvas_left(layer.x(dot.at) - Plot::Tick / 2.0)
                    .canvas_top(y_px(dot.y) - Plot::Radius)
                    .width(Plot::Tick)
                    .height(Plot::Radius * 2.0)
                    .fill(if dot.faint { faint(palette.critical) } else { palette.critical })
                    .into(),
            ));
        }
        let start = scatter.now.saturating_sub(scatter.length);
        for (at, clock) in ticks(scatter.now, scatter.now_clock, scatter.length) {
            if at < start {
                continue;
            }
            items.push((
                format!("tick/{at}"),
                caption(format::clock(clock))
                    .foreground(palette.secondary_text)
                    .canvas_left(layer.x(at))
                    .canvas_top(Plot::Height + space::Compact)
                    .into(),
            ));
        }
        Canvas::new().children((View::keyed_fragment(items),))
    }
}

fn key(dot: &Dot) -> String {
    format!("{}:{}", dot.key.pid, dot.key.sequence)
}

#[derive(Clone, PartialEq)]
pub struct Plotted {
    pub scatter: Rc<Scatter>,
    pub card: Option<(ProcessInstance, View)>,
    pub palette: Palette,
    pub on_pick: Callback<Area>,
    pub on_clear: Callback<()>,
    pub on_hover: Callback<Option<ProcessInstance>>,
}

pub enum Pointer {
    Pressed(PointerEventInfo),
    Moved(PointerEventInfo),
    Released(PointerEventInfo),
    Left,
    Sized(f64),
    Frame,
}

pub struct ScatterPlot {
    input: Plotted,
    arrived: Instant,
    origin: u64,
    width: f64,
    pressed: Option<(f64, f64)>,
    dragged: Option<(f64, f64)>,
    hovered: Option<(ProcessInstance, f64, f64)>,
    host: ElementRef<Grid>,
    _sized: ElementObservation,
    _frame: Option<ComponentTimer>,
}

impl ScatterPlot {
    fn scale(&self) -> Scale {
        let elapsed = self.arrived.elapsed().as_nanos() / 100;
        Scale {
            now: self.input.scatter.now + elapsed as u64,
            length: self.input.scatter.length.max(1),
            width: self.width,
        }
    }

    fn hover(&mut self, at: Option<(f64, f64)>) {
        let scale = self.scale();
        let found = at.and_then(|at| nearest(&self.input.scatter, &scale, at));
        if found.map(|(key, ..)| key) != self.hovered.map(|(key, ..)| key) {
            let _ = self.input.on_hover.call(found.map(|(key, ..)| key));
        }
        self.hovered = found;
    }

    fn card(&self) -> View {
        let shown = self
            .hovered
            .zip(self.input.card.as_ref())
            .filter(|((key, ..), (card_key, _))| key == card_key);
        let Some(((_, x, y), (_, card))) = shown else {
            return View::empty();
        };
        let left = (x + Plot::Offset).min(self.width - Plot::Card).max(0.0);
        Canvas::new().children((Border::new()
            .canvas_left(left)
            .canvas_top(y + Plot::Offset)
            .width(Plot::Card)
            .content(card.clone()),))
    }

    fn overlay(&self, scale: &Scale) -> View {
        let accent = accent_color();
        let mut items: Vec<(String, View)> = Vec::new();
        if let Some((key, ..)) = self.hovered
            && let Some(dot) = self.input.scatter.dots.iter().chain(&self.input.scatter.orphans).find(|dot| dot.key == key)
        {
            let (x, y) = (scale.x(dot.at), y_px(dot.y));
            items.push((
                "hovered".into(),
                Ellipse::new()
                    .canvas_left(x - Plot::Ring)
                    .canvas_top(y - Plot::Ring)
                    .width(Plot::Ring * 2.0)
                    .height(Plot::Ring * 2.0)
                    .stroke(accent)
                    .stroke_thickness(1.5)
                    .into(),
            ));
        }
        if let Some((from, to)) = self.pressed.zip(self.dragged) {
            items.push(("dragged".into(), marked(from.0, to.0, from.1, to.1, accent)));
        }
        Canvas::new().children((View::keyed_fragment(items),))
    }

    fn grid(&self) -> View {
        let lines: Vec<(String, View)> = self
            .input
            .scatter
            .levels
            .iter()
            .map(|level| {
                (
                    level.lived.to_string(),
                    Border::new()
                        .height(0.5)
                        .vertical_alignment(VerticalAlignment::Top)
                        .margin(Thickness::new(0.0, y_px(level.y), 0.0, 0.0))
                        .background(self.input.palette.divider_stroke)
                        .into(),
                )
            })
            .collect();
        Grid::new().height(Plot::Height).children((View::keyed_fragment(lines),))
    }
}

impl Component for ScatterPlot {
    type Input = Plotted;
    type Message = Pointer;

    fn create(input: &Plotted, cx: &ComponentContext<Self>) -> Self {
        let host = ElementRef::new();
        let sender = cx.sender();
        let sized = host.observe_composition_host(move |event| {
            let width = match event {
                CompositionHostEvent::Ready { width, .. } | CompositionHostEvent::Metrics { width, .. } => width,
            };
            let _ = sender.send(Pointer::Sized(width));
        });
        Self {
            input: input.clone(),
            arrived: Instant::now(),
            origin: input.scatter.now.saturating_sub(input.scatter.length),
            width: Plot::Assumed,
            pressed: None,
            dragged: None,
            hovered: None,
            host,
            _sized: sized,
            _frame: cx.set_timeout(Plot::Frame, Pointer::Frame).ok(),
        }
    }

    fn input_changed(&mut self, input: &Plotted, _cx: &ComponentContext<Self>) {
        if input.scatter.length != self.input.scatter.length {
            self.origin = input.scatter.now.saturating_sub(input.scatter.length);
        }
        self.input = input.clone();
        self.arrived = Instant::now();
    }

    fn update(&mut self, message: Pointer, cx: &ComponentContext<Self>) {
        match message {
            Pointer::Frame => {
                self._frame = cx.set_timeout(Plot::Frame, Pointer::Frame).ok();
            }
            Pointer::Sized(width) => {
                if width > 0.0 {
                    self.width = width;
                }
            }
            Pointer::Pressed(info) => {
                self.pressed = Some((info.x, info.y));
                self.dragged = None;
            }
            Pointer::Moved(info) => match self.pressed {
                Some(_) => {
                    self.dragged = Some((info.x, info.y));
                    self.hover(None);
                }
                None => self.hover(Some((info.x, info.y))),
            },
            Pointer::Released(info) => {
                let Some(from) = self.pressed.take() else {
                    return;
                };
                self.dragged = None;
                let to = (info.x, info.y);
                if (to.0 - from.0).abs() < Plot::Click && (to.1 - from.1).abs() < Plot::Click {
                    if self.input.scatter.area.is_some() {
                        let _ = self.input.on_clear.call(());
                    }
                } else {
                    let area = self.scale().area(from.0, to.0, y_of(from.1.max(to.1)), y_of(from.1.min(to.1)));
                    let _ = self.input.on_pick.call(area);
                }
            }
            Pointer::Left => {
                if self.pressed.is_none() {
                    self.hover(None);
                }
            }
        }
    }

    fn view(&self, input: &Plotted, cx: &mut ViewContext<Self>) -> View {
        let scale = self.scale();
        let per_tick = scale.width / scale.length as f64;
        let layer = Layer {
            scatter: input.scatter.clone(),
            origin: self.origin,
            per_tick,
            palette: input.palette,
        };
        let offset = scale.width - (scale.now as f64 - self.origin as f64) * per_tick;
        Grid::new()
            .element_ref(&self.host)
            .height(Plot::Height + Plot::Axis)
            .children((
                self.grid(),
                Canvas::new().children((Border::new()
                    .canvas_left(offset)
                    .content(View::component::<Dots>(layer)),)),
                self.overlay(&scale),
                Border::new()
                    .mark(ActivityMark::Scatter)
                    .height(Plot::Height)
                    .vertical_alignment(VerticalAlignment::Top)
                    .background(Color::transparent())
                    .capture_pointer_on_press(true)
                    .on_pointer_pressed(cx.callback(Pointer::Pressed))
                    .on_pointer_moved(cx.callback(Pointer::Moved))
                    .on_pointer_released(cx.callback(Pointer::Released))
                    .on_pointer_exited(cx.callback(|_: PointerEventInfo| Pointer::Left)),
                self.card(),
            ))
    }
}

pub fn labels(scatter: &Scatter, label: impl Fn(u64) -> String, palette: Palette) -> View {
    let items: Vec<(String, View)> = scatter
        .levels
        .iter()
        .map(|level| {
            (
                level.lived.to_string(),
                caption(label(level.lived))
                    .foreground(palette.secondary_text)
                    .canvas_left(0.0)
                    .canvas_top(y_px(level.y) - space::Control)
                    .into(),
            )
        })
        .collect();
    Canvas::new()
        .width(Plot::Labels)
        .height(Plot::Height)
        .children((View::keyed_fragment(items),))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(pid: u32, at: u64, y: f32) -> Dot {
        Dot {
            key: ProcessInstance { pid, sequence: 1 },
            at,
            y,
            alive: false,
            faint: false,
        }
    }

    #[test]
    fn the_pointer_finds_the_closest_dot_within_reach_and_none_beyond() {
        let second = super::super::timeline::Ticks::Second;
        let scale = Scale { now: 4000 * second, length: 400 * second, width: 400.0 };
        let scatter = Scatter {
            dots: vec![at(1, 3700 * second, 0.5), at(2, 3705 * second, 0.5)],
            orphans: vec![at(3, 3900 * second, -0.08)],
            ..Scatter::default()
        };

        let pid = |point: (f64, f64)| nearest(&scatter, &scale, point).map(|(key, ..)| key.pid);

        assert_eq!(
            (
                pid((100.0, y_px(0.5))),
                pid((105.0, y_px(0.5))),
                pid((300.0, y_px(-0.08) + 2.0)),
                pid((200.0, y_px(0.5)))
            ),
            (Some(1), Some(2), Some(3), None)
        );
    }
}
