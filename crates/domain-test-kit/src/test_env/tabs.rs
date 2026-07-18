use app_contracts::features::tabs::{UiTabsBindings, UiTabsPort, UiTabsPortMsg};
use forsl_core::test_kit::Interaction;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Default)]
struct TabsStubState {
    messages: RefCell<Vec<UiTabsPortMsg>>,
    on_request_tab_switch: RefCell<Option<Box<dyn Fn(String)>>>,
    on_request_tab_close: RefCell<Option<Box<dyn Fn(String)>>>,
    on_request_tab_add: RefCell<Option<Box<dyn Fn(String)>>>,
}

#[derive(Clone, Default)]
pub struct TabsPortStub {
    inner: Rc<TabsStubState>,
}

impl TabsPortStub {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn all(&self) -> Vec<UiTabsPortMsg> {
        self.inner.messages.borrow().clone()
    }

    pub fn emit_request_tab_switch(&self, context_key: String) -> Interaction<()> {
        self.inner
            .on_request_tab_switch
            .borrow()
            .as_ref()
            .expect("on_request_tab_switch not registered")(context_key);
        Interaction::new(())
    }

    pub fn emit_request_tab_close(&self, context_key: String) -> Interaction<()> {
        self.inner
            .on_request_tab_close
            .borrow()
            .as_ref()
            .expect("on_request_tab_close not registered")(context_key);
        Interaction::new(())
    }

    pub fn emit_request_tab_add(&self, context_key: String) -> Interaction<()> {
        self.inner
            .on_request_tab_add
            .borrow()
            .as_ref()
            .expect("on_request_tab_add not registered")(context_key);
        Interaction::new(())
    }
}

impl UiTabsPort for TabsPortStub {
    fn send(&self, msg: UiTabsPortMsg) {
        self.inner.messages.borrow_mut().push(msg);
    }
}

impl UiTabsBindings for TabsPortStub {
    fn on_request_tab_switch<F>(&self, handler: F)
    where
        F: Fn(String) + 'static,
    {
        *self.inner.on_request_tab_switch.borrow_mut() = Some(Box::new(handler));
    }

    fn on_request_tab_close<F>(&self, handler: F)
    where
        F: Fn(String) + 'static,
    {
        *self.inner.on_request_tab_close.borrow_mut() = Some(Box::new(handler));
    }

    fn on_request_tab_add<F>(&self, handler: F)
    where
        F: Fn(String) + 'static,
    {
        *self.inner.on_request_tab_add.borrow_mut() = Some(Box::new(handler));
    }
}
