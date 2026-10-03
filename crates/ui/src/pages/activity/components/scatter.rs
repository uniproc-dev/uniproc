use std::cell::Cell;
use std::rc::Rc;

use app_contracts::features::activity::{Area, Dot, Scatter};
use app_contracts::features::agents::ProcessInstance;
use guinea::winui::MarkExt;
use windows_canvas::{canvas, ColorF, DrawContext, Ellipse, Rect, Vector2};
use windows_reactor::{
    Border, Callback, CanvasChildExt, ChildrenControl, Color, Component, ComponentContext, ContentControl, Grid,
    GridChildExt, GridLength, HorizontalAlignment, LayoutControl, PointerEventInfo, Thickness, View, ViewContext,
};

use super::super::marks::ActivityMark;
use crate::format;
use crate::theme::{accent_color, space, Palette};
use crate::widgets::text::caption;

pub struct Plot;

#[expect(non_upper_case_globals)]
impl Plot {
    pub const Height: f64 = 126.0;
    const Assumed: f64 = 800.0;
    const Top: f64 = 8.0;
    const Bottom: f64 = 6.0;
    const Lowest: f32 = -0.14;
    const Highest: f32 = 1.06;
    const Radius: f32 = 2.6;
    const Ring: f32 = 4.5;
    const Reach: f64 = 8.0;
    const Click: f64 = 4.0;
    const Card: f64 = 320.0;
    const Offset: f64 = 14.0;
    const Labels: f64 = 56.0;
    const AreaAlpha: u8 = 40;
}

fn y_px(y: f32) -> f64 {
    let share = f64::from(Plot::Highest - y) / f64::from(Plot::Highest - Plot::Lowest);
    Plot::Top + share * (Plot::Height - Plot::Top - Plot::Bottom)
}

fn y_of(px: f64) -> f32 {
    let share = ((px - Plot::Top) / (Plot::Height - Plot::Top - Plot::Bottom)) as f32;
    Plot::Highest - share * (Plot::Highest - Plot::Lowest)
}

pub fn area_of(from: (f64, f64), to: (f64, f64), width: f64) -> Area {
    let x = |px: f64| (px / width).clamp(0.0, 1.0) as f32;
    Area {
        left: x(from.0.min(to.0)),
        right: x(from.0.max(to.0)),
        bottom: y_of(from.1.max(to.1)),
        top: y_of(from.1.min(to.1)),
    }
}

pub fn nearest(scatter: &Scatter, at: (f64, f64), width: f64) -> Option<(ProcessInstance, f64, f64)> {
    scatter
        .dots
        .iter()
        .chain(&scatter.orphans)
        .map(|dot| {
            let (x, y) = (f64::from(dot.x) * width, y_px(dot.y));
            (dot.key, x, y, (x - at.0).hypot(y - at.1))
        })
        .filter(|(.., distance)| *distance <= Plot::Reach)
        .min_by(|a, b| a.3.total_cmp(&b.3))
        .map(|(key, x, y, _)| (key, x, y))
}

fn paint(color: Color) -> ColorF {
    ColorF::from_rgba8(color.r, color.g, color.b, color.a)
}

fn faint(color: Color, alpha: u8) -> ColorF {
    ColorF::from_rgba8(color.r, color.g, color.b, alpha)
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
}

pub struct ScatterPlot {
    input: Plotted,
    width: Rc<Cell<f64>>,
    pressed: Option<(f64, f64)>,
    dragged: Option<(f64, f64)>,
    hovered: Option<(ProcessInstance, f64, f64)>,
}

impl ScatterPlot {
    fn hover(&mut self, at: Option<(f64, f64)>) {
        let found = at.and_then(|at| nearest(&self.input.scatter, at, self.width.get()));
        if found.map(|(key, ..)| key) != self.hovered.map(|(key, ..)| key) {
            let _ = self.input.on_hover.call(found.map(|(key, ..)| key));
        }
        self.hovered = found;
    }

    fn draw(&self) -> impl Fn(&DrawContext<'_>) -> windows_canvas::Result<()> + 'static {
        let scatter = self.input.scatter.clone();
        let palette = self.input.palette;
        let width = self.width.clone();
        let hovered = self.hovered.map(|(key, ..)| key);
        let dragged = self.pressed.zip(self.dragged);
        move |cx: &DrawContext<'_>| {
            width.set(f64::from(cx.width));
            cx.clear(ColorF::TRANSPARENT);
            let w = cx.width;
            let px = |dot: &Dot| Vector2::new(dot.x * w, y_px(dot.y) as f32);

            let grid = cx.create_solid_brush(paint(palette.divider_stroke))?;
            for level in &scatter.levels {
                let y = y_px(level.y) as f32;
                cx.draw_line(Vector2::new(0.0, y), Vector2::new(w, y), &grid, 0.5);
            }
            let base = y_px(0.0) as f32 + Plot::Ring;
            cx.draw_line(Vector2::new(0.0, base), Vector2::new(w, base), &grid, 1.0);

            let accent = accent_color();
            let marked = |area: &Area| {
                Rect::new(
                    area.left * w,
                    y_px(area.top) as f32,
                    area.right * w,
                    y_px(area.bottom) as f32,
                )
            };
            if let Some(area) = &scatter.area {
                cx.fill_rect(&marked(area), &cx.create_solid_brush(faint(accent, Plot::AreaAlpha))?);
                cx.draw_rect(&marked(area), &cx.create_solid_brush(paint(accent))?, 1.0);
            }

            let came = cx.create_solid_brush(paint(palette.success))?;
            for dot in &scatter.dots {
                let circle = Ellipse::circle(px(dot), Plot::Radius);
                if dot.alive {
                    cx.draw_ellipse(&circle, &came, 1.2);
                } else {
                    cx.fill_ellipse(&circle, &came);
                }
            }
            let went = cx.create_solid_brush(paint(palette.critical))?;
            for dot in &scatter.orphans {
                let at = px(dot);
                cx.draw_line(
                    Vector2::new(at.x, at.y - Plot::Radius),
                    Vector2::new(at.x, at.y + Plot::Radius),
                    &went,
                    1.4,
                );
            }

            if let Some(key) = hovered
                && let Some(dot) = scatter.dots.iter().chain(&scatter.orphans).find(|dot| dot.key == key)
            {
                let ring = cx.create_solid_brush(paint(accent))?;
                cx.draw_ellipse(&Ellipse::circle(px(dot), Plot::Ring), &ring, 1.5);
            }
            if let Some((from, to)) = dragged {
                let rect = Rect::new(
                    from.0.min(to.0) as f32,
                    from.1.min(to.1) as f32,
                    from.0.max(to.0) as f32,
                    from.1.max(to.1) as f32,
                );
                cx.fill_rect(&rect, &cx.create_solid_brush(faint(accent, Plot::AreaAlpha))?);
                cx.draw_rect(&rect, &cx.create_solid_brush(paint(accent))?, 1.0);
            }
            Ok(())
        }
    }

    fn card(&self) -> View {
        let shown = self
            .hovered
            .zip(self.input.card.as_ref())
            .filter(|((key, ..), (card_key, _))| key == card_key);
        let Some(((_, x, y), (_, card))) = shown else {
            return View::empty();
        };
        let width = self.width.get();
        let left = (x + Plot::Offset).min(width - Plot::Card).max(0.0);
        windows_reactor::Canvas::new().children((Border::new()
            .canvas_left(left)
            .canvas_top(y + Plot::Offset)
            .width(Plot::Card)
            .content(card.clone()),))
    }
}

impl Component for ScatterPlot {
    type Input = Plotted;
    type Message = Pointer;

    fn create(input: &Plotted, _cx: &ComponentContext<Self>) -> Self {
        Self {
            input: input.clone(),
            width: Rc::new(Cell::new(Plot::Assumed)),
            pressed: None,
            dragged: None,
            hovered: None,
        }
    }

    fn input_changed(&mut self, input: &Plotted, _cx: &ComponentContext<Self>) {
        self.input = input.clone();
    }

    fn update(&mut self, message: Pointer, _cx: &ComponentContext<Self>) {
        match message {
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
                    let _ = self.input.on_pick.call(area_of(from, to, self.width.get()));
                }
            }
            Pointer::Left => {
                if self.pressed.is_none() {
                    self.hover(None);
                }
            }
        }
    }

    fn view(&self, _input: &Plotted, cx: &mut ViewContext<Self>) -> View {
        Grid::new()
            .height(Plot::Height)
            .children((
                canvas(self.draw()),
                Border::new()
                    .mark(ActivityMark::Scatter)
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

pub fn labels(scatter: &Scatter, label: impl Fn(u64) -> String, running: String, before: String, palette: Palette) -> View {
    let placed = |key: String, text: String, y: f32| {
        (
            key,
            caption(text)
                .foreground(palette.secondary_text)
                .canvas_left(0.0)
                .canvas_top(y_px(y) - space::Control)
                .into(),
        )
    };
    let mut items: Vec<(String, View)> = scatter
        .levels
        .iter()
        .map(|level| placed(level.lived.to_string(), label(level.lived), level.y))
        .collect();
    items.push(placed("running".into(), running, scatter.alive_y));
    if let Some(orphan) = scatter.orphans.first() {
        items.push(placed("before".into(), before, orphan.y));
    }
    windows_reactor::Canvas::new()
        .width(Plot::Labels)
        .height(Plot::Height)
        .children((View::keyed_fragment(items),))
}

pub fn ticks(scatter: &Scatter, palette: Palette) -> View {
    let last = scatter.ticks.len().saturating_sub(1);
    let items: Vec<(String, View)> = scatter
        .ticks
        .iter()
        .enumerate()
        .map(|(at, tick)| {
            let alignment = if at == last && at > 0 {
                HorizontalAlignment::Right
            } else {
                HorizontalAlignment::Left
            };
            (
                at.to_string(),
                caption(format::clock(*tick))
                    .grid_column(at.min(last.saturating_sub(1)) as i32)
                    .horizontal_alignment(alignment)
                    .foreground(palette.secondary_text)
                    .into(),
            )
        })
        .collect();
    Grid::new()
        .columns(vec![GridLength::Star(1.0); last.max(1)])
        .margin(Thickness::new(0.0, space::Compact, 0.0, 0.0))
        .children((View::keyed_fragment(items),))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(pid: u32, x: f32, y: f32) -> Dot {
        Dot {
            key: ProcessInstance { pid, sequence: 1 },
            x,
            y,
            alive: false,
        }
    }

    #[test]
    fn the_pointer_finds_the_closest_dot_within_reach_and_none_beyond() {
        let scatter = Scatter {
            dots: vec![at(1, 0.25, 0.5), at(2, 0.26, 0.5)],
            orphans: vec![at(3, 0.75, -0.08)],
            ..Scatter::default()
        };
        let width = 400.0;

        let near_first = nearest(&scatter, (100.0, y_px(0.5)), width).map(|(key, ..)| key.pid);
        let near_second = nearest(&scatter, (105.0, y_px(0.5)), width).map(|(key, ..)| key.pid);
        let near_orphan = nearest(&scatter, (300.0, y_px(-0.08) + 2.0), width).map(|(key, ..)| key.pid);
        let far = nearest(&scatter, (200.0, y_px(0.5)), width);

        assert_eq!((near_first, near_second, near_orphan, far), (Some(1), Some(2), Some(3), None));
    }

    #[test]
    fn a_dragged_rectangle_becomes_the_area_it_covers() {
        let area = area_of((300.0, y_px(0.2)), (100.0, y_px(0.6)), 400.0);
        let close = |a: f32, b: f32| (a - b).abs() < 0.001;
        assert!(close(area.left, 0.25) && close(area.right, 0.75), "{area:?}");
        assert!(close(area.bottom, 0.2) && close(area.top, 0.6), "{area:?}");
    }
}
