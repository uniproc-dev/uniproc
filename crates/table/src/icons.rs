use std::sync::Arc;

use windows_canvas::{Bitmap, ColorF, DrawingSession, GpuDevice, Rect};
use windows_core::{Interface, Result};

use crate::bindings as c;
use crate::cache::{premultiplied_bgra, Cache};
use crate::model::Icon;

struct Held {
    _pixels: Option<Arc<[u8]>>,
    bitmap: Bitmap,
}

pub(crate) struct Icons {
    cache: Cache<(usize, u32), Held>,
}

#[expect(non_upper_case_globals)]
impl Icons {
    const Kept: usize = 512;
}

impl Icons {
    pub(crate) fn new() -> Self {
        Self { cache: Cache::new(Self::Kept) }
    }

    pub(crate) fn draw(
        &mut self,
        device: &GpuDevice,
        session: &DrawingSession<'_>,
        icon: &Icon,
        dest: &Rect,
        scale: f32,
        opacity: f32,
    ) -> Result<()> {
        let pixels = match icon {
            Icon::Svg(_) => ((dest.right - dest.left) * scale).round().max(1.0) as u32,
            Icon::Rgba { .. } => 0,
        };
        let key = (icon.identity(), pixels);
        if self.cache.get(&key).is_none() {
            let held = make(device, session, icon, pixels)?;
            self.cache.insert(key, held);
        }
        if let Some(held) = self.cache.get(&key) {
            session.draw_bitmap(&held.bitmap, dest, opacity);
        }
        Ok(())
    }

    pub(crate) fn sweep(&mut self) {
        self.cache.sweep();
    }
}

fn make(device: &GpuDevice, session: &DrawingSession<'_>, icon: &Icon, pixels: u32) -> Result<Held> {
    match icon {
        Icon::Svg(svg) => {
            let target = device.create_render_target(pixels, pixels)?;
            target.draw(|offscreen| {
                offscreen.clear(ColorF::from_rgba8(0, 0, 0, 0));
                let context: c::ID2D1DeviceContext5 = offscreen.raw().cast()?;
                let stream = unsafe { c::SHCreateMemStream(Some(svg)) }
                    .ok_or_else(|| windows_core::Error::from_hresult(windows_core::HRESULT(0x8007000Eu32 as i32)))?;
                unsafe {
                    let document = context.CreateSvgDocument(
                        &stream,
                        c::D2D_SIZE_F { width: pixels as f32, height: pixels as f32 },
                    )?;
                    context.DrawSvgDocument(&document);
                }
                Ok(())
            })?;
            let bgra = target.read_pixels()?;
            Ok(Held { _pixels: None, bitmap: session.create_bitmap(&bgra, pixels, pixels)? })
        }
        Icon::Rgba { width, height, pixels } => Ok(Held {
            bitmap: session.create_bitmap(&premultiplied_bgra(pixels), *width, *height)?,
            _pixels: Some(pixels.clone()),
        }),
    }
}
