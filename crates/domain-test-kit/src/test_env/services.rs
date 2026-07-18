use app_contracts::features::services::{
    ServiceActionKind, ServiceEntryVm, ServicesWindowRegister, UiServiceDetailsPort,
    UiServiceDetailsPortMsg, UiServicesBindings, UiServicesPort, UiServicesPortMsg,
};
use forsl::native_windows::slint_factory::SlintWindowRegistry;
use slint::SharedString;
use std::cell::RefCell;
use std::rc::Rc;

type HandlerStrKind = Box<dyn Fn(SharedString, ServiceActionKind)>;
type HandlerStr = Box<dyn Fn(SharedString)>;
type HandlerStrI32 = Box<dyn Fn(SharedString, i32)>;
type HandlerI2 = Box<dyn Fn(i32, i32)>;
type HandlerStrF32 = Box<dyn Fn(SharedString, f32)>;
type HandlerEntry = Box<dyn Fn(ServiceEntryVm)>;

#[derive(Default)]
struct ServicesStubState {
    details_messages: RefCell<Vec<UiServiceDetailsPortMsg>>,
    messages: RefCell<Vec<UiServicesPortMsg>>,
    on_service_action: RefCell<Option<HandlerStrKind>>,
    on_sort_by: RefCell<Option<HandlerStr>>,
    on_select_service: RefCell<Option<HandlerStrI32>>,
    on_rows_viewport_changed: RefCell<Option<HandlerI2>>,
    on_column_resized: RefCell<Option<HandlerStrF32>>,
    on_open_properties_window: RefCell<Option<HandlerEntry>>,
}

#[derive(Clone, Default)]
pub struct ServicesPortStub {
    inner: Rc<ServicesStubState>,
}

impl std::fmt::Debug for ServicesPortStub {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ServicesPortStub").finish()
    }
}

impl ServicesPortStub {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn all(&self) -> Vec<UiServicesPortMsg> {
        self.inner.messages.borrow().clone()
    }

    pub fn all_details(&self) -> Vec<UiServiceDetailsPortMsg> {
        self.inner.details_messages.borrow().clone()
    }

    pub fn emit_select_service(&self, name: SharedString, idx: i32) {
        self.inner
            .on_select_service
            .borrow()
            .as_ref()
            .expect("on_select_service not registered")(name, idx);
    }
}

impl UiServiceDetailsPort for ServicesPortStub {
    fn send(&self, msg: UiServiceDetailsPortMsg) {
        self.inner.details_messages.borrow_mut().push(msg);
    }
}

impl UiServicesPort for ServicesPortStub {
    fn send(&self, msg: UiServicesPortMsg) {
        self.inner.messages.borrow_mut().push(msg);
    }
}

impl ServicesWindowRegister for ServicesPortStub {
    fn register(&self, _registry: &SlintWindowRegistry) {}
}

impl UiServicesBindings for ServicesPortStub {
    fn on_service_action<F>(&self, handler: F)
    where
        F: Fn(SharedString, ServiceActionKind) + 'static,
    {
        *self.inner.on_service_action.borrow_mut() = Some(Box::new(handler));
    }

    fn on_sort_by<F>(&self, handler: F)
    where
        F: Fn(SharedString) + 'static,
    {
        *self.inner.on_sort_by.borrow_mut() = Some(Box::new(handler));
    }

    fn on_select_service<F>(&self, handler: F)
    where
        F: Fn(SharedString, i32) + 'static,
    {
        *self.inner.on_select_service.borrow_mut() = Some(Box::new(handler));
    }

    fn on_rows_viewport_changed<F>(&self, handler: F)
    where
        F: Fn(i32, i32) + 'static,
    {
        *self.inner.on_rows_viewport_changed.borrow_mut() = Some(Box::new(handler));
    }

    fn on_column_resized<F>(&self, handler: F)
    where
        F: Fn(SharedString, f32) + 'static,
    {
        *self.inner.on_column_resized.borrow_mut() = Some(Box::new(handler));
    }

    fn on_open_properties_window<F>(&self, handler: F)
    where
        F: Fn(ServiceEntryVm) + 'static,
    {
        *self.inner.on_open_properties_window.borrow_mut() = Some(Box::new(handler));
    }
}
