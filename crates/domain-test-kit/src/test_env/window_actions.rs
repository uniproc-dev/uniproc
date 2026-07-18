use app_contracts::features::window_actions::{
    ResizeEdge, UiWindowActionsBindings, UiWindowActionsPort, UiWindowActionsPortMsg,
    WindowBreakpoint,
};
use forsl_core::test_kit::Interaction;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Default)]
struct WindowActionsStubState {
    messages: RefCell<Vec<UiWindowActionsPortMsg>>,
    on_drag: RefCell<Option<Box<dyn Fn()>>>,
    on_close: RefCell<Option<Box<dyn Fn()>>>,
    on_minimize: RefCell<Option<Box<dyn Fn()>>>,
    on_maximize: RefCell<Option<Box<dyn Fn()>>>,
    on_start_resize: RefCell<Option<Box<dyn Fn(ResizeEdge)>>>,
    on_config_changed: RefCell<Option<Box<dyn Fn(WindowBreakpoint, u64)>>>,
}

#[derive(Clone, Default)]
pub struct WindowActionsPortStub {
    inner: Rc<WindowActionsStubState>,
}

impl WindowActionsPortStub {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn all(&self) -> Vec<UiWindowActionsPortMsg> {
        self.inner.messages.borrow().clone()
    }

    pub fn emit_drag(&self) -> Interaction<()> {
        self.inner.on_drag.borrow().as_ref().expect("on_drag not registered")();
        Interaction::new(())
    }

    pub fn emit_close(&self) -> Interaction<()> {
        self.inner.on_close.borrow().as_ref().expect("on_close not registered")();
        Interaction::new(())
    }

    pub fn emit_minimize(&self) -> Interaction<()> {
        self.inner.on_minimize.borrow().as_ref().expect("on_minimize not registered")();
        Interaction::new(())
    }

    pub fn emit_maximize(&self) -> Interaction<()> {
        self.inner.on_maximize.borrow().as_ref().expect("on_maximize not registered")();
        Interaction::new(())
    }

    pub fn emit_start_resize(&self, edge: ResizeEdge) -> Interaction<()> {
        self.inner
            .on_start_resize
            .borrow()
            .as_ref()
            .expect("on_start_resize not registered")(edge);
        Interaction::new(())
    }
}

impl UiWindowActionsPort for WindowActionsPortStub {
    fn send(&self, msg: UiWindowActionsPortMsg) {
        self.inner.messages.borrow_mut().push(msg);
    }
}

impl UiWindowActionsBindings for WindowActionsPortStub {
    fn on_start_resize<F>(&self, handler: F)
    where
        F: Fn(ResizeEdge) + 'static,
    {
        *self.inner.on_start_resize.borrow_mut() = Some(Box::new(handler));
    }

    fn on_config_changed<F>(&self, handler: F)
    where
        F: Fn(WindowBreakpoint, u64) + 'static,
    {
        *self.inner.on_config_changed.borrow_mut() = Some(Box::new(handler));
    }

    fn on_drag<F>(&self, handler: F)
    where
        F: Fn() + 'static,
    {
        *self.inner.on_drag.borrow_mut() = Some(Box::new(handler));
    }

    fn on_close<F>(&self, handler: F)
    where
        F: Fn() + 'static,
    {
        *self.inner.on_close.borrow_mut() = Some(Box::new(handler));
    }

    fn on_minimize<F>(&self, handler: F)
    where
        F: Fn() + 'static,
    {
        *self.inner.on_minimize.borrow_mut() = Some(Box::new(handler));
    }

    fn on_maximize<F>(&self, handler: F)
    where
        F: Fn() + 'static,
    {
        *self.inner.on_maximize.borrow_mut() = Some(Box::new(handler));
    }
}
