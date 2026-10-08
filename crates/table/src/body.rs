use std::cell::{Cell, RefCell};
use std::rc::Rc;

use windows_reactor::{
    keyed, Border, Callback, Color, Component, ComponentContext, CompositionHostEvent, ElementObservation, ElementRef,
    Grid, HorizontalAlignment, KeyEventInfo, PointerEventInfo, RoutedCallback, Thickness, Tooltip, TooltipExt,
    TooltipPlacement, VerticalAlignment, View, ViewContext, VirtualKey,
};

use crate::layout::Width;
use crate::model::{RowKey, Source};
use crate::nav::Step;
use crate::paint::Look;
use crate::scene::{BarHit, Scene};

#[derive(Clone)]
pub struct Body {
    pub source: Rc<dyn Source>,
    pub widths: Rc<[Width]>,
    pub look: Look,
    pub selected: Option<RowKey>,
    pub on_select: Option<Callback<Option<RowKey>>>,
}

impl PartialEq for Body {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.source, &other.source)
            && self.widths == other.widths
            && self.look == other.look
            && self.selected == other.selected
            && self.on_select == other.on_select
    }
}

pub fn body(body: Body) -> View {
    View::component::<Painted>(body)
}

pub enum Msg {
    Scrolled,
    Moved(f32, f32),
    Left,
    Pressed(f32, f32),
    Released,
    Key(Step),
}

fn step_of(key: VirtualKey) -> Option<Step> {
    match key {
        VirtualKey::UP => Some(Step::Up),
        VirtualKey::DOWN => Some(Step::Down),
        VirtualKey::PAGE_UP => Some(Step::PageUp),
        VirtualKey::PAGE_DOWN => Some(Step::PageDown),
        VirtualKey::HOME => Some(Step::Home),
        VirtualKey::END => Some(Step::End),
        _ => None,
    }
}

struct Tip {
    key: RowKey,
    column: usize,
    text: String,
    bounds: [f32; 4],
}

pub struct Painted {
    keys: RoutedCallback<KeyEventInfo>,
    host: ElementRef<Grid>,
    _observation: ElementObservation,
    scene: Rc<RefCell<Option<Scene>>>,
    input: Rc<RefCell<Body>>,
    hovered: Rc<Cell<Option<f32>>>,
    pointer: f32,
    grabbed: Option<f32>,
    tip: Option<Tip>,
}

fn frame(scene: &mut Scene, input: &Body, hovered: Option<f32>) {
    if let Err(error) = scene.frame(&*input.source, hovered, input.selected) {
        tracing::warn!(%error, "table frame");
    }
}

fn tip_view(tip: &Tip) -> View {
    let [x, y, width, height] = tip.bounds;
    Border::new()
        .background(Color::transparent())
        .horizontal_alignment(HorizontalAlignment::Left)
        .vertical_alignment(VerticalAlignment::Top)
        .margin(Thickness::new(x as f64, y as f64, 0.0, 0.0))
        .width(width as f64)
        .height(height as f64)
        .tooltip_with(Tooltip::text(&tip.text).placement(TooltipPlacement::Mouse))
}

fn tip(scene: &Scene, source: &dyn Source, x: f32, y: Option<f32>) -> Option<Tip> {
    let (at, column, bounds) = scene.cell(x, y?)?;
    let text = source.tip(at, column)?;
    Some(Tip { key: source.key(at), column, text, bounds })
}

impl Component for Painted {
    type Input = Body;
    type Message = Msg;

    fn create(input: &Body, cx: &ComponentContext<Self>) -> Self {
        let host = ElementRef::<Grid>::new();
        let scene: Rc<RefCell<Option<Scene>>> = Rc::new(RefCell::new(None));
        let shared_input = Rc::new(RefCell::new(input.clone()));
        let hovered = Rc::new(Cell::new(None));
        let attach = host.clone();
        let built = scene.clone();
        let current = shared_input.clone();
        let pointed = hovered.clone();
        let sender = cx.sender();
        let observation = host.observe_composition_host(move |event| {
            let input = current.borrow();
            match event {
                CompositionHostEvent::Ready { compositor, width, height, scale } => {
                    let sender = sender.clone();
                    let wake = Box::new(move || {
                        let _ = sender.send(Msg::Scrolled);
                    });
                    let made = Scene::new(compositor, &input.look, wake).and_then(|mut scene| {
                        scene.style(&input.widths, &input.look)?;
                        scene.resize(width as f32, height as f32, scale as f32)?;
                        Ok(scene)
                    });
                    match made {
                        Ok(mut made) => {
                            match made.root() {
                                Ok(root) => {
                                    let _ = attach.request_set_child_visual(Some(root), |result| {
                                        if let Err(error) = result {
                                            tracing::warn!(?error, "table attach");
                                        }
                                    });
                                }
                                Err(error) => tracing::warn!(%error, "table root"),
                            }
                            frame(&mut made, &input, pointed.get());
                            *built.borrow_mut() = Some(made);
                        }
                        Err(error) => tracing::warn!(%error, "table scene"),
                    }
                }
                CompositionHostEvent::Metrics { width, height, scale } => {
                    if let Some(scene) = built.borrow_mut().as_mut() {
                        if let Err(error) = scene.resize(width as f32, height as f32, scale as f32) {
                            tracing::warn!(%error, "table resize");
                        }
                        frame(scene, &input, pointed.get());
                    }
                }
            }
        });
        let keyed = cx.sender();
        let keys = RoutedCallback::new(move |info: KeyEventInfo| match step_of(info.key) {
            Some(step) => {
                let _ = keyed.send(Msg::Key(step));
                true
            }
            None => false,
        });
        Self {
            keys,
            host,
            _observation: observation,
            scene,
            input: shared_input,
            hovered,
            pointer: 0.0,
            grabbed: None,
            tip: None,
        }
    }

    fn input_changed(&mut self, input: &Body, _cx: &ComponentContext<Self>) {
        *self.input.borrow_mut() = input.clone();
        if let Some(scene) = self.scene.borrow_mut().as_mut() {
            if let Err(error) = scene.style(&input.widths, &input.look) {
                tracing::warn!(%error, "table style");
            }
            frame(scene, input, self.hovered.get());
            self.tip = tip(scene, &*input.source, self.pointer, self.hovered.get());
        }
    }

    fn update(&mut self, message: Msg, _cx: &ComponentContext<Self>) {
        let mut scene = self.scene.borrow_mut();
        let Some(scene) = scene.as_mut() else {
            return;
        };
        let input = self.input.borrow();
        match message {
            Msg::Scrolled => {
                frame(scene, &input, self.hovered.get());
                self.tip = tip(scene, &*input.source, self.pointer, self.hovered.get());
            }
            Msg::Moved(x, y) => {
                if let Some(grab) = self.grabbed {
                    if let Err(error) = scene.drag_bar(y, grab) {
                        tracing::warn!(%error, "table drag");
                    }
                    return;
                }
                let over_bar = scene.bar_hit(x, y).is_some();
                let hovered = (!over_bar).then_some(y);
                self.hovered.set(hovered);
                self.pointer = x;
                let _ = scene.widen_bar(over_bar);
                let _ = scene.point(hovered);
                self.tip = tip(scene, &*input.source, x, hovered);
            }
            Msg::Left => {
                self.hovered.set(None);
                self.tip = None;
                let _ = scene.point(None);
                if self.grabbed.is_none() {
                    let _ = scene.widen_bar(false);
                }
            }
            Msg::Pressed(x, y) => match scene.bar_hit(x, y) {
                Some(BarHit::Thumb(grab)) => self.grabbed = Some(grab),
                Some(BarHit::Above) => {
                    let _ = scene.page(false);
                }
                Some(BarHit::Below) => {
                    let _ = scene.page(true);
                }
                None => {
                    if let Some(on_select) = &input.on_select {
                        on_select.call(scene.at(y).map(|at| input.source.key(at)));
                    }
                }
            },
            Msg::Released => self.grabbed = None,
            Msg::Key(step) => {
                let source = &*input.source;
                let current = input.selected.and_then(|key| (0..source.len()).find(|&at| source.key(at) == key));
                if let Some(at) = scene.step(current, step) {
                    if let Some(on_select) = &input.on_select {
                        on_select.call(Some(source.key(at)));
                    }
                    if let Err(error) = scene.reveal(at) {
                        tracing::warn!(%error, "table reveal");
                    }
                }
            }
        }
    }

    fn view(&self, _input: &Body, cx: &mut ViewContext<Self>) -> View {
        Border::new()
            .is_tab_stop(true)
            .focus_on_pointer_release(true)
            .on_preview_key_down(self.keys.clone())
            .background(Color::transparent())
            .content(Grid::new().keyed_children(
                std::iter::once(keyed("host".to_string(), View::from(Grid::new().element_ref(&self.host))))
                    .chain(self.tip.as_ref().map(|tip| keyed(format!("tip/{}/{}", tip.key, tip.column), tip_view(tip)))),
            ))
            .capture_pointer_on_press(true)
            .on_pointer_moved(cx.callback(|event: PointerEventInfo| Msg::Moved(event.x as f32, event.y as f32)))
            .on_pointer_exited(cx.callback(|_: PointerEventInfo| Msg::Left))
            .on_pointer_pressed(cx.callback(|event: PointerEventInfo| Msg::Pressed(event.x as f32, event.y as f32)))
            .on_pointer_released(cx.callback(|_: PointerEventInfo| Msg::Released))
            .on_pointer_capture_lost(cx.callback(|_: ()| Msg::Released))
            .into()
    }
}
