use windows_canvas::{Brush, ColorF, DrawingSession, GpuDevice, Rect, RoundedRect};
use windows_core::Result;
use windows_numerics::Vector2;

use crate::bindings as c;
use crate::icons::Icons;
use crate::layout::{self, Lines};
use crate::model::{Cell, Chevron, Rgba, Tone};
use crate::text::{target, Line, Text};

#[derive(Clone, Debug, PartialEq)]
pub struct Look {
    pub tones: [Rgba; 5],
    pub hovered: Rgba,
    pub selected: Rgba,
    pub plate_inset: (f32, f32),
    pub plate_radius: f32,
    pub cell_inset: f32,
    pub heat_inset: f32,
    pub heat_radius: f32,
    pub dim: f32,
    pub chevron: f32,
    pub icon: f32,
    pub icon_gap: f32,
    pub note_gap: f32,
    pub fonts: &'static [&'static str],
    pub font_size: f32,
    pub thumb: Rgba,
    pub thumb_thin: f32,
    pub thumb_wide: f32,
    pub thumb_min: f32,
    pub bar_margin: f32,
    pub bar_zone: f32,
}

impl Default for Look {
    fn default() -> Self {
        let grey = |a| Rgba { r: 128, g: 128, b: 128, a };
        let white = |a| Rgba { r: 255, g: 255, b: 255, a };
        Self {
            tones: [white(255), white(200), white(139), white(93), Rgba { r: 108, g: 203, b: 95, a: 255 }],
            hovered: grey(24),
            selected: grey(40),
            plate_inset: (4.0, 1.0),
            plate_radius: 4.0,
            cell_inset: 12.0,
            heat_inset: 4.0,
            heat_radius: 4.0,
            dim: 0.55,
            chevron: 16.0,
            icon: 16.0,
            icon_gap: 8.0,
            note_gap: 4.0,
            fonts: &["Segoe UI Variable Text", "Segoe UI"],
            font_size: 12.0,
            thumb: white(139),
            thumb_thin: 2.0,
            thumb_wide: 6.0,
            thumb_min: 24.0,
            bar_margin: 3.0,
            bar_zone: 14.0,
        }
    }
}

impl Look {
    pub fn tone(&self, tone: Tone) -> Rgba {
        self.tones[match tone {
            Tone::Primary => 0,
            Tone::Secondary => 1,
            Tone::Tertiary => 2,
            Tone::Disabled => 3,
            Tone::Success => 4,
        }]
    }
}

fn color(rgba: Rgba, dim: f32) -> ColorF {
    ColorF::from_rgba8(rgba.r, rgba.g, rgba.b, (rgba.a as f32 * dim).round() as u8)
}

pub struct Row<'a> {
    pub cells: &'a [Cell],
    pub columns: &'a Lines,
    pub width: f32,
    pub height: f32,
}

pub(crate) struct Ink<'a> {
    pub text: &'a Text,
    pub look: &'a Look,
    pub icons: &'a mut Icons,
    pub device: &'a GpuDevice,
    pub scale: f32,
}

pub(crate) fn row(session: &DrawingSession<'_>, ink: &mut Ink<'_>, row: &Row<'_>) -> Result<()> {
    session.clear(ColorF::from_rgba8(0, 0, 0, 0));
    let target = target(session);
    for (at, cell) in row.cells.iter().enumerate().take(row.columns.len()) {
        let (left, right) = layout::column_span(row.columns, row.width, at);
        let clip = c::D2D_RECT_F { left, top: 0.0, right, bottom: row.height };
        unsafe { target.PushAxisAlignedClip(&clip, c::D2D1_ANTIALIAS_MODE_PER_PRIMITIVE) };
        let painted = paint_cell(session, ink, cell, left, right, row.height);
        unsafe { target.PopAxisAlignedClip() };
        painted?;
    }
    Ok(())
}

fn paint_cell(session: &DrawingSession<'_>, ink: &mut Ink<'_>, cell: &Cell, left: f32, right: f32, height: f32) -> Result<()> {
    let (text, look) = (ink.text, ink.look);
    let dim = if cell.dim { look.dim } else { 1.0 };
    if let Some(heat) = cell.heat.filter(|heat| heat.a > 0) {
        let wash = session.create_solid_brush(color(heat, dim))?;
        let inset = look.heat_inset;
        let plate = Rect::from_xywh(left + inset, inset, (right - left - 2.0 * inset).max(0.0), (height - 2.0 * inset).max(0.0));
        session.fill_rounded_rect(&RoundedRect::uniform(plate, look.heat_radius), &wash);
    }
    let mut x = left + look.cell_inset + cell.indent;
    let right = right - look.cell_inset;
    if cell.chevron != Chevron::None {
        if cell.chevron != Chevron::Slot {
            let ink = session.create_solid_brush(color(look.tone(Tone::Secondary), dim))?;
            chevron(session, &ink, x + look.chevron / 2.0, height / 2.0, cell.chevron == Chevron::Expanded);
        }
        x += look.chevron;
    }
    if let Some(icon) = &cell.icon {
        let top = ((height - look.icon) / 2.0).round();
        let dest = Rect::from_xywh(x, top, look.icon, look.icon);
        ink.icons.draw(ink.device, session, icon, &dest, ink.scale, dim)?;
        x += look.icon + look.icon_gap;
    }
    if x >= right {
        return Ok(());
    }
    let ink = session.create_solid_brush(color(look.tone(cell.tone), dim))?;
    if cell.note.is_empty() {
        text.draw(session, &line(&cell.text, cell, x, right, height), &ink);
        return Ok(());
    }
    let note_width = text.width(cell.weight, &cell.note).unwrap_or(0.0);
    let main_right = (right - note_width - look.note_gap).max(x);
    let drawn = text.draw(session, &line(&cell.text, cell, x, main_right, height), &ink);
    let note_left = drawn.map_or(main_right, |width| x + width) + look.note_gap;
    let note_ink: Brush = session.create_solid_brush(color(look.tone(Tone::Tertiary), dim))?;
    let note = Line { align: crate::model::Align::Start, ..line(&cell.note, cell, note_left, right, height) };
    text.draw(session, &note, &note_ink);
    Ok(())
}

fn line<'a>(text: &'a str, cell: &Cell, left: f32, right: f32, height: f32) -> Line<'a> {
    Line { text, weight: cell.weight, align: cell.align, left, right, top: 0.0, height }
}

fn chevron(session: &DrawingSession<'_>, ink: &Brush, x: f32, y: f32, open: bool) {
    let arm = 3.0;
    let (a, b, tip) = if open {
        (Vector2::new(x - arm, y - arm / 2.0), Vector2::new(x + arm, y - arm / 2.0), Vector2::new(x, y + arm / 2.0))
    } else {
        (Vector2::new(x - arm / 2.0, y - arm), Vector2::new(x - arm / 2.0, y + arm), Vector2::new(x + arm / 2.0, y))
    };
    session.draw_line(a, tip, ink, 1.0);
    session.draw_line(b, tip, ink, 1.0);
}
