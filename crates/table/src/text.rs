use std::cell::RefCell;
use std::collections::HashMap;
use std::mem::ManuallyDrop;

use windows_canvas::{DrawingSession, Rect, TextFormat, WordWrapping};
use windows_core::{Interface, Result, BOOL, HSTRING};
use windows_numerics::Vector2;

use crate::bindings as c;
use crate::model::{Align, Weight};
use crate::trim::{trim, Fit};

const ELLIPSIS: char = '\u{2026}';

struct Face {
    face: c::IDWriteFontFace,
    scale: f32,
    ascent: f32,
    descent: f32,
    ascii: [(u16, f32); 128],
    other: RefCell<HashMap<char, (u16, f32)>>,
}

impl Face {
    fn new(family: &c::IDWriteFontFamily, weight: c::DWRITE_FONT_WEIGHT, size: f32) -> Result<Self> {
        unsafe {
            let font = family.GetFirstMatchingFont(weight, c::DWRITE_FONT_STRETCH_NORMAL, c::DWRITE_FONT_STYLE_NORMAL)?;
            let face = font.CreateFontFace()?;
            let mut metrics = c::DWRITE_FONT_METRICS::default();
            face.GetMetrics(&mut metrics);
            let scale = size / metrics.designUnitsPerEm as f32;
            let mut built = Self {
                face,
                scale,
                ascent: metrics.ascent as f32 * scale,
                descent: metrics.descent as f32 * scale,
                ascii: [(0, 0.0); 128],
                other: RefCell::new(HashMap::new()),
            };
            let points: Vec<u32> = (0..128).collect();
            let glyphs = built.lookup(&points)?;
            built.ascii.copy_from_slice(&glyphs);
            Ok(built)
        }
    }

    fn lookup(&self, points: &[u32]) -> Result<Vec<(u16, f32)>> {
        let mut glyphs = vec![0u16; points.len()];
        let mut metrics = vec![c::DWRITE_GLYPH_METRICS::default(); points.len()];
        unsafe {
            self.face.GetGlyphIndices(points.as_ptr(), points.len() as u32, glyphs.as_mut_ptr()).ok()?;
            self.face
                .GetDesignGlyphMetrics(glyphs.as_ptr(), glyphs.len() as u32, metrics.as_mut_ptr(), false)
                .ok()?;
        }
        Ok(glyphs
            .into_iter()
            .zip(metrics)
            .map(|(glyph, metrics)| (glyph, metrics.advanceWidth as f32 * self.scale))
            .collect())
    }

    fn glyph(&self, ch: char) -> Option<(u16, f32)> {
        let found = if (ch as u32) < 128 {
            self.ascii[ch as usize]
        } else {
            let known = self.other.borrow().get(&ch).copied();
            match known {
                Some(found) => found,
                None => {
                    let found = self.lookup(&[ch as u32]).ok()?[0];
                    self.other.borrow_mut().insert(ch, found);
                    found
                }
            }
        };
        (found.0 != 0).then_some(found)
    }
}

fn simple(ch: char) -> bool {
    matches!(ch as u32, 0x20..0x300 | 0x370..0x530 | 0x1E00..0x2200)
}

struct Run {
    indices: Vec<u16>,
    advances: Vec<f32>,
}

pub struct Text {
    faces: [Face; 2],
    formats: [TextFormat; 4],
    size: f32,
    run: RefCell<Run>,
}

pub struct Line<'a> {
    pub text: &'a str,
    pub weight: Weight,
    pub align: Align,
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub height: f32,
}

impl Text {
    pub fn new(families: &[&str], size: f32) -> Result<Self> {
        unsafe {
            let factory: c::IDWriteFactory = c::DWriteCreateFactory(c::DWRITE_FACTORY_TYPE_SHARED)?;
            let mut collection = None;
            factory.GetSystemFontCollection(&mut collection, false).ok()?;
            let collection = collection.ok_or_else(|| windows_core::Error::from_hresult(windows_core::HRESULT(-1)))?;
            let mut chosen = None;
            for name in families {
                let mut index = 0;
                let mut exists = BOOL(0);
                collection.FindFamilyName(&HSTRING::from(*name), &mut index, &mut exists).ok()?;
                if exists.as_bool() {
                    chosen = Some((*name, collection.GetFontFamily(index)?));
                    break;
                }
            }
            let (name, family) = chosen.ok_or_else(|| windows_core::Error::from_hresult(windows_core::HRESULT(-1)))?;
            let faces = [
                Face::new(&family, c::DWRITE_FONT_WEIGHT_NORMAL, size)?,
                Face::new(&family, c::DWRITE_FONT_WEIGHT_SEMI_BOLD, size)?,
            ];
            let format = |weight: c::DWRITE_FONT_WEIGHT, align: windows_canvas::TextAlignment| -> Result<TextFormat> {
                let format = TextFormat::with_weight(name, size, windows_canvas::CanvasFontWeight(weight))?
                    .with_alignment(align)
                    .with_paragraph_alignment(windows_canvas::ParagraphAlignment::Center)
                    .with_word_wrapping(WordWrapping::NoWrap);
                let raw: c::IDWriteTextFormat = format.raw().cast()?;
                let sign = factory.CreateEllipsisTrimmingSign(&raw)?;
                let trimming = c::DWRITE_TRIMMING {
                    granularity: c::DWRITE_TRIMMING_GRANULARITY_CHARACTER,
                    delimiter: 0,
                    delimiterCount: 0,
                };
                raw.SetTrimming(&trimming, &sign).ok()?;
                Ok(format)
            };
            use windows_canvas::TextAlignment::{Leading, Trailing};
            let formats = [
                format(c::DWRITE_FONT_WEIGHT_NORMAL, Leading)?,
                format(c::DWRITE_FONT_WEIGHT_NORMAL, Trailing)?,
                format(c::DWRITE_FONT_WEIGHT_SEMI_BOLD, Leading)?,
                format(c::DWRITE_FONT_WEIGHT_SEMI_BOLD, Trailing)?,
            ];
            Ok(Self {
                faces,
                formats,
                size,
                run: RefCell::new(Run {
                    indices: Vec::new(),
                    advances: Vec::new(),
                }),
            })
        }
    }

    fn face(&self, weight: Weight) -> &Face {
        match weight {
            Weight::Normal => &self.faces[0],
            Weight::Strong => &self.faces[1],
        }
    }

    pub fn width(&self, weight: Weight, text: &str) -> Option<f32> {
        let face = self.face(weight);
        text.chars()
            .map(|ch| simple(ch).then(|| face.glyph(ch)).flatten().map(|(_, advance)| advance))
            .sum()
    }

    pub fn draw(&self, session: &DrawingSession<'_>, line: &Line<'_>, brush: &windows_canvas::Brush) -> Option<f32> {
        let drawn = self.draw_glyphs(session, line, brush);
        if drawn.is_none() {
            let format = &self.formats[match (line.weight, line.align) {
                (Weight::Normal, Align::Start) => 0,
                (Weight::Normal, Align::End) => 1,
                (Weight::Strong, Align::Start) => 2,
                (Weight::Strong, Align::End) => 3,
            }];
            let rect = Rect::from_xywh(line.left, line.top, (line.right - line.left).max(0.0), line.height);
            session.draw_text(line.text, format, &rect, brush);
        }
        drawn
    }

    fn draw_glyphs(&self, session: &DrawingSession<'_>, line: &Line<'_>, brush: &windows_canvas::Brush) -> Option<f32> {
        let face = self.face(line.weight);
        let mut run = self.run.borrow_mut();
        let Run { indices, advances } = &mut *run;
        indices.clear();
        advances.clear();
        for ch in line.text.chars() {
            if !simple(ch) {
                return None;
            }
            let (glyph, advance) = face.glyph(ch)?;
            indices.push(glyph);
            advances.push(advance);
        }
        let room = line.right - line.left;
        let ellipsis = face.glyph(ELLIPSIS);
        if let Fit::Cut(kept) = trim(advances, room, ellipsis.map_or(0.0, |(_, advance)| advance)) {
            indices.truncate(kept);
            advances.truncate(kept);
            if let Some((glyph, advance)) = ellipsis {
                indices.push(glyph);
                advances.push(advance);
            }
        }
        let width: f32 = advances.iter().sum();
        if indices.is_empty() {
            return Some(width);
        }
        let x = match line.align {
            Align::Start => line.left,
            Align::End => line.right - width,
        };
        let baseline = line.top + (line.height - face.ascent - face.descent) / 2.0 + face.ascent;
        let glyph_run = c::DWRITE_GLYPH_RUN {
            fontFace: ManuallyDrop::new(Some(face.face.clone())),
            fontEmSize: self.size,
            glyphCount: indices.len() as u32,
            glyphIndices: indices.as_ptr(),
            glyphAdvances: advances.as_ptr(),
            glyphOffsets: core::ptr::null(),
            isSideways: false.into(),
            bidiLevel: 0,
        };
        let target = target(session);
        let brush = brush_of(brush);
        unsafe {
            target.DrawGlyphRun(Vector2::new(x, baseline), &glyph_run, brush, c::DWRITE_MEASURING_MODE_NATURAL);
        }
        drop(ManuallyDrop::into_inner(glyph_run.fontFace));
        Some(width)
    }
}

pub(crate) fn target<'a>(session: &'a DrawingSession<'_>) -> &'a c::ID2D1RenderTarget {
    let raw = session.raw();
    unsafe { c::ID2D1RenderTarget::from_raw_borrowed(&*(raw as *const windows_canvas::ID2D1DeviceContext as *const *mut core::ffi::c_void)) }
        .expect("a device context is a render target")
}

fn brush_of(brush: &windows_canvas::Brush) -> &c::ID2D1Brush {
    use windows_canvas::Paint;
    let raw = brush.as_raw_brush();
    unsafe { c::ID2D1Brush::from_raw_borrowed(&*(raw as *const _ as *const *mut core::ffi::c_void)) }.expect("a brush")
}
