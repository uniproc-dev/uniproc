use std::cell::Cell as Slot;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;

use windows_canvas::{DrawingSession, GpuDevice, ID2D1DeviceContext, Matrix3x2};
use windows_core::{IUnknown, Interface, Result, HSTRING};
use windows_numerics::{Vector2, Vector3};

use crate::bindings as c;
use crate::icons::Icons;
use crate::interop::{ICompositionDrawingSurfaceInterop, ICompositorInterop, Point, Size};
use crate::layout::{self, Band, Lines, Width};
use crate::model::{Cell, RowKey, Source};
use crate::paint::{self, Look};
use crate::realize::{Realized, Visuals};
use crate::text::Text;

pub(crate) struct Shared {
    position: AtomicU32,
    pending: AtomicBool,
    below: AtomicU32,
    above: AtomicU32,
}

impl Shared {
    fn new() -> Self {
        Self {
            position: AtomicU32::new(0f32.to_bits()),
            pending: AtomicBool::new(false),
            below: AtomicU32::new(Band::ALWAYS.below.to_bits()),
            above: AtomicU32::new(Band::ALWAYS.above.to_bits()),
        }
    }

    fn position(&self) -> f32 {
        f32::from_bits(self.position.load(Ordering::Relaxed))
    }

    fn band(&self) -> Band {
        Band {
            below: f32::from_bits(self.below.load(Ordering::Relaxed)),
            above: f32::from_bits(self.above.load(Ordering::Relaxed)),
        }
    }

    fn hold(&self, band: Band) {
        self.below.store(band.below.to_bits(), Ordering::Relaxed);
        self.above.store(band.above.to_bits(), Ordering::Relaxed);
    }
}

#[windows_core::implement(c::IInteractionTrackerOwner)]
struct Owner {
    shared: Arc<Shared>,
    wake: Box<dyn Fn()>,
}

impl c::IInteractionTrackerOwner_Impl for Owner_Impl {
    fn CustomAnimationStateEntered(
        &self,
        _: windows_core::Ref<c::InteractionTracker>,
        _: windows_core::Ref<c::InteractionTrackerCustomAnimationStateEnteredArgs>,
    ) -> Result<()> {
        Ok(())
    }

    fn IdleStateEntered(
        &self,
        _: windows_core::Ref<c::InteractionTracker>,
        _: windows_core::Ref<c::InteractionTrackerIdleStateEnteredArgs>,
    ) -> Result<()> {
        Ok(())
    }

    fn InertiaStateEntered(
        &self,
        _: windows_core::Ref<c::InteractionTracker>,
        _: windows_core::Ref<c::InteractionTrackerInertiaStateEnteredArgs>,
    ) -> Result<()> {
        Ok(())
    }

    fn InteractingStateEntered(
        &self,
        _: windows_core::Ref<c::InteractionTracker>,
        _: windows_core::Ref<c::InteractionTrackerInteractingStateEnteredArgs>,
    ) -> Result<()> {
        Ok(())
    }

    fn RequestIgnored(
        &self,
        _: windows_core::Ref<c::InteractionTracker>,
        _: windows_core::Ref<c::InteractionTrackerRequestIgnoredArgs>,
    ) -> Result<()> {
        Ok(())
    }

    fn ValuesChanged(
        &self,
        _: windows_core::Ref<c::InteractionTracker>,
        args: windows_core::Ref<c::InteractionTrackerValuesChangedArgs>,
    ) -> Result<()> {
        let position = args.ok()?.Position()?.y;
        self.shared.position.store(position.to_bits(), Ordering::Relaxed);
        if !self.shared.band().holds(position) && !self.shared.pending.swap(true, Ordering::Relaxed) {
            (self.wake)();
        }
        Ok(())
    }
}

struct Plate {
    sprite: c::SpriteVisual,
    brush: c::CompositionColorBrush,
    geometry: c::CompositionRoundedRectangleGeometry,
}

impl Plate {
    fn new(compositor: &c::Compositor) -> Result<Self> {
        let sprite = compositor.CreateSpriteVisual()?;
        let brush = compositor.CreateColorBrushWithColor(c::Color::default())?;
        sprite.SetBrush(&brush)?;
        let geometry = compositor.CreateRoundedRectangleGeometry()?;
        sprite.SetClip(&compositor.CreateGeometricClipWithGeometry(&geometry)?)?;
        sprite.SetIsVisible(false)?;
        Ok(Self { sprite, brush, geometry })
    }

    fn show(&self, top: f32, width: f32, height: f32, look: &Look, rgba: crate::model::Rgba) -> Result<()> {
        let (across, down) = look.plate_inset;
        self.brush.SetColor(c::Color { A: rgba.a, R: rgba.r, G: rgba.g, B: rgba.b })?;
        self.sprite.SetOffset(Vector3::new(0.0, top, 0.0))?;
        self.sprite.SetSize(Vector2::new(width, height))?;
        self.geometry.SetOffset(Vector2::new(across, down))?;
        self.geometry.SetSize(Vector2::new((width - 2.0 * across).max(0.0), (height - 2.0 * down).max(0.0)))?;
        self.geometry.SetCornerRadius(Vector2::new(look.plate_radius, look.plate_radius))?;
        self.sprite.SetIsVisible(true)
    }

    fn hide(&self) -> Result<()> {
        self.sprite.SetIsVisible(false)
    }
}

pub(crate) struct RowVisual {
    sprite: c::SpriteVisual,
    surface: Option<ICompositionDrawingSurfaceInterop>,
    pixels: Slot<(i32, i32)>,
    offset: Slot<f32>,
    size: Slot<(f32, f32)>,
}

struct Painter {
    compositor: c::Compositor,
    graphics: c::CompositionGraphicsDevice,
    device: GpuDevice,
    content: c::ContainerVisual,
    icons: Icons,
    scale: f32,
    width: f32,
    lines: Lines,
    columns: Lines,
    text: Text,
    look: Look,
    error: Option<windows_core::Error>,
}

impl Painter {
    fn row_width(&self) -> f32 {
        self.width.max(self.columns.extent())
    }

    fn fail(&mut self, result: Result<()>) {
        if let Err(error) = result {
            self.error.get_or_insert(error);
        }
    }

    fn make(&self) -> Result<RowVisual> {
        let sprite = self.compositor.CreateSpriteVisual()?;
        let surface = self.graphics.CreateDrawingSurface(
            c::Size { Width: 1.0, Height: 1.0 },
            c::DirectXPixelFormat::B8G8R8A8UIntNormalized,
            c::DirectXAlphaMode::Premultiplied,
        )?;
        sprite.SetBrush(&self.compositor.CreateSurfaceBrushWithSurface(&surface.cast::<c::ICompositionSurface>()?)?)?;
        self.content.Children()?.InsertAtTop(&sprite)?;
        Ok(RowVisual {
            sprite,
            surface: Some(surface.cast()?),
            pixels: Slot::new((1, 1)),
            offset: Slot::new(f32::NAN),
            size: Slot::new((0.0, 0.0)),
        })
    }

    fn settle(&self, visual: &RowVisual, at: usize) -> Result<()> {
        let offset = self.lines.start(at);
        if visual.offset.get() != offset {
            visual.sprite.SetOffset(Vector3::new(0.0, offset, 0.0))?;
            visual.offset.set(offset);
        }
        let size = (self.row_width(), self.lines.size(at));
        if visual.size.get() != size {
            visual.sprite.SetSize(Vector2::new(size.0, size.1))?;
            visual.size.set(size);
        }
        Ok(())
    }

    fn paint(&mut self, visual: &RowVisual, cells: &[Cell]) -> Result<()> {
        let Some(surface) = &visual.surface else {
            return Ok(());
        };
        let (width, height) = visual.size.get();
        let pixels = ((width * self.scale).ceil().max(1.0) as i32, (height * self.scale).ceil().max(1.0) as i32);
        if visual.pixels.get() != pixels {
            unsafe { surface.Resize(Size { cx: pixels.0, cy: pixels.1 }).ok()? };
            visual.pixels.set(pixels);
        }
        let mut offset = Point::default();
        let context: ID2D1DeviceContext = unsafe { surface.BeginDraw(None, &mut offset)? };
        let painted = {
            let at = Matrix3x2::translation(offset.x as f32 / self.scale, offset.y as f32 / self.scale);
            let session = DrawingSession::from_borrowed_context_with_dpi(&context, at, 96.0 * self.scale);
            let row = paint::Row { cells, columns: &self.columns, width, height };
            let mut ink = paint::Ink {
                text: &self.text,
                look: &self.look,
                icons: &mut self.icons,
                device: &self.device,
                scale: self.scale,
            };
            paint::row(&session, &mut ink, &row)
        };
        let ended = unsafe { surface.EndDraw().ok() };
        painted.and(ended)
    }
}

impl Visuals for Painter {
    type Visual = RowVisual;

    fn create(&mut self) -> RowVisual {
        match self.make() {
            Ok(visual) => visual,
            Err(error) => {
                self.error.get_or_insert(error);
                RowVisual {
                    sprite: self.compositor.CreateSpriteVisual().expect("a sprite visual"),
                    surface: None,
                    pixels: Slot::new((0, 0)),
                    offset: Slot::new(f32::NAN),
                    size: Slot::new((0.0, 0.0)),
                }
            }
        }
    }

    fn retire(&mut self, visual: &RowVisual) {
        let hidden = visual.sprite.SetIsVisible(false);
        self.fail(hidden);
    }

    fn revive(&mut self, visual: &RowVisual) {
        let shown = visual.sprite.SetIsVisible(true);
        self.fail(shown);
    }

    fn place(&mut self, visual: &RowVisual, at: usize) {
        let settled = self.settle(visual, at);
        self.fail(settled);
    }

    fn draw(&mut self, _key: RowKey, visual: &RowVisual, cells: &[Cell], _changed: &[bool]) {
        let painted = self.paint(visual, cells);
        self.fail(painted);
    }
}

pub(crate) struct Scene {
    root: c::ContainerVisual,
    tracker: c::InteractionTracker,
    _source: c::VisualInteractionSource,
    shared: Arc<Shared>,
    hover: Plate,
    selection: Plate,
    rows: Realized<RowVisual>,
    painter: Painter,
    widths: Vec<Width>,
    height: f32,
    extent: f32,
    overscan: f32,
}

impl Scene {
    pub(crate) fn new(compositor: IUnknown, look: &Look, wake: Box<dyn Fn()>) -> Result<Self> {
        let compositor: c::Compositor = compositor.cast()?;
        let device = GpuDevice::new_or_warp()?;
        let interop: ICompositorInterop = compositor.cast()?;
        let graphics: c::CompositionGraphicsDevice = unsafe { interop.CreateGraphicsDevice(&device.d2d_device().cast()?)? }.cast()?;
        let root = compositor.CreateContainerVisual()?;
        let content = compositor.CreateContainerVisual()?;
        root.SetClip(&compositor.CreateInsetClip()?)?;
        root.Children()?.InsertAtTop(&content)?;
        let hover = Plate::new(&compositor)?;
        let selection = Plate::new(&compositor)?;
        content.Children()?.InsertAtBottom(&hover.sprite)?;
        content.Children()?.InsertAtBottom(&selection.sprite)?;
        let shared = Arc::new(Shared::new());
        let owner: c::IInteractionTrackerOwner = Owner { shared: shared.clone(), wake }.into();
        let tracker = c::InteractionTracker::CreateWithOwner(&compositor, &owner)?;
        let source = c::VisualInteractionSource::Create(&root)?;
        source.SetManipulationRedirectionMode(c::VisualInteractionSourceRedirectionMode::CapableTouchpadAndPointerWheel)?;
        source.SetPositionYSourceMode(c::InteractionSourceMode::EnabledWithInertia)?;
        tracker.InteractionSources()?.Add(&source.cast::<c::ICompositionInteractionSource>()?)?;
        let follow = compositor.CreateExpressionAnimationWithExpression(&HSTRING::from("-tracker.Position"))?;
        follow.SetReferenceParameter(&HSTRING::from("tracker"), &tracker)?;
        content.StartAnimation(&HSTRING::from("Offset"), &follow)?;
        let text = Text::new(look.fonts, look.font_size)?;
        Ok(Self {
            root,
            tracker,
            _source: source,
            shared,
            hover,
            selection,
            rows: Realized::default(),
            painter: Painter {
                compositor,
                graphics,
                device,
                content,
                icons: Icons::new(),
                scale: 1.0,
                width: 0.0,
                lines: Lines::default(),
                columns: Lines::default(),
                text,
                look: look.clone(),
                error: None,
            },
            widths: Vec::new(),
            height: 0.0,
            extent: -1.0,
            overscan: 256.0,
        })
    }

    pub(crate) fn root(&self) -> Result<IUnknown> {
        self.root.cast()
    }

    pub(crate) fn resize(&mut self, width: f32, height: f32, scale: f32) -> Result<()> {
        self.root.SetSize(Vector2::new(width, height))?;
        self.height = height;
        self.extent = -1.0;
        if self.painter.width != width || self.painter.scale != scale {
            self.painter.width = width;
            self.painter.scale = scale;
            self.painter.columns = layout::columns(&self.widths, width);
            self.rows.forget();
        }
        Ok(())
    }

    pub(crate) fn style(&mut self, widths: &[Width], look: &Look) -> Result<()> {
        if self.widths != widths {
            self.widths = widths.to_vec();
            self.painter.columns = layout::columns(&self.widths, self.painter.width);
            self.rows.forget();
        }
        if self.painter.look != *look {
            if self.painter.look.fonts != look.fonts || self.painter.look.font_size != look.font_size {
                self.painter.text = Text::new(look.fonts, look.font_size)?;
            }
            self.painter.look = look.clone();
            self.rows.forget();
        }
        Ok(())
    }

    pub(crate) fn at(&self, y: f32) -> Option<usize> {
        self.painter.lines.at(y + self.shared.position())
    }

    pub(crate) fn frame(&mut self, source: &dyn Source, hovered: Option<f32>, selected: Option<RowKey>) -> Result<()> {
        if self.painter.width <= 0.0 || self.height <= 0.0 {
            return Ok(());
        }
        self.painter.lines.refill((0..source.len()).map(|at| source.height(at)));
        let extent = (self.painter.lines.extent() - self.height).max(0.0);
        if extent != self.extent {
            self.tracker.SetMaxPosition(Vector3::new(0.0, extent, 0.0))?;
            self.extent = extent;
        }
        self.shared.pending.store(false, Ordering::Relaxed);
        let position = self.shared.position().clamp(0.0, extent);
        let range = layout::realized(&self.painter.lines, position, self.height, self.overscan);
        self.rows.update(source, range.clone(), &mut self.painter);
        self.painter.icons.sweep();
        for (_, row) in self.rows.rows() {
            let settled = self.painter.settle(&row.visual, row.at);
            self.painter.fail(settled);
        }
        self.shared.hold(layout::band(&self.painter.lines, range, self.height, self.overscan / 2.0));
        self.point(hovered)?;
        let width = self.painter.row_width();
        let look = &self.painter.look;
        match selected.and_then(|key| self.rows.get(key)) {
            Some(row) => {
                let at = row.at;
                self.selection.show(self.painter.lines.start(at), width, self.painter.lines.size(at), look, look.selected)?
            }
            None => self.selection.hide()?,
        }
        match self.painter.error.take() {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }

    pub(crate) fn point(&self, hovered: Option<f32>) -> Result<()> {
        let position = self.shared.position();
        let look = &self.painter.look;
        match hovered.and_then(|y| self.painter.lines.at(y + position)) {
            Some(at) => self.hover.show(
                self.painter.lines.start(at),
                self.painter.row_width(),
                self.painter.lines.size(at),
                look,
                look.hovered,
            ),
            None => self.hover.hide(),
        }
    }
}
