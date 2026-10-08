use std::cell::{Cell, RefCell};
use std::rc::Rc;

use windows_reactor::{
    Border, Callback, Color, Component, ComponentContext, CompositionHostEvent, ElementObservation, ElementRef, Grid,
    KeyEventInfo, PointerEventInfo, RoutedCallback, View, ViewContext, VirtualKey,
};

use crate::layout::Width;
use crate::model::{RowKey, Source};
use crate::nav::Step;
use crate::paint::Look;
use crate::scene::Scene;

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
    Moved(f32),
    Left,
    Pressed(f32),
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

pub struct Painted {
    keys: RoutedCallback<KeyEventInfo>,
    host: ElementRef<Grid>,
    _observation: ElementObservation,
    scene: Rc<RefCell<Option<Scene>>>,
    input: Rc<RefCell<Body>>,
    hovered: Rc<Cell<Option<f32>>>,
}

fn frame(scene: &mut Scene, input: &Body, hovered: Option<f32>) {
    if let Err(error) = scene.frame(&*input.source, hovered, input.selected) {
        tracing::warn!(%error, "table frame");
    }
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
        }
    }

    fn input_changed(&mut self, input: &Body, _cx: &ComponentContext<Self>) {
        *self.input.borrow_mut() = input.clone();
        if let Some(scene) = self.scene.borrow_mut().as_mut() {
            if let Err(error) = scene.style(&input.widths, &input.look) {
                tracing::warn!(%error, "table style");
            }
            frame(scene, input, self.hovered.get());
        }
    }

    fn update(&mut self, message: Msg, _cx: &ComponentContext<Self>) {
        let mut scene = self.scene.borrow_mut();
        let Some(scene) = scene.as_mut() else {
            return;
        };
        let input = self.input.borrow();
        match message {
            Msg::Scrolled => frame(scene, &input, self.hovered.get()),
            Msg::Moved(y) => {
                self.hovered.set(Some(y));
                let _ = scene.point(Some(y));
            }
            Msg::Left => {
                self.hovered.set(None);
                let _ = scene.point(None);
            }
            Msg::Pressed(y) => {
                if let Some(on_select) = &input.on_select {
                    on_select.call(scene.at(y).map(|at| input.source.key(at)));
                }
            }
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
            .content(Grid::new().element_ref(&self.host))
            .on_pointer_moved(cx.callback(|event: PointerEventInfo| Msg::Moved(event.y as f32)))
            .on_pointer_exited(cx.callback(|_: PointerEventInfo| Msg::Left))
            .on_pointer_pressed(cx.callback(|event: PointerEventInfo| Msg::Pressed(event.y as f32)))
            .into()
    }
}
