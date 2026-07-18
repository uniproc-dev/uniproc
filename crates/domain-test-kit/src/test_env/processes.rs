use app_contracts::features::processes::{UiProcessesBindings, UiProcessesPort, UiProcessesPortMsg};
use slint::SharedString;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

type Handler0 = Box<dyn Fn()>;
type HandlerStr = Box<dyn Fn(SharedString)>;
type HandlerI2 = Box<dyn Fn(i32, i32)>;
type HandlerStrF32 = Box<dyn Fn(SharedString, f32)>;

#[derive(Default)]
struct ProcessesStubState {
    messages: RefCell<Vec<UiProcessesPortMsg>>,
    selected_pid: Cell<i32>,
    on_sort_by: RefCell<Option<HandlerStr>>,
    on_toggle_expand_group: RefCell<Option<HandlerStr>>,
    on_terminate: RefCell<Option<Handler0>>,
    on_select_process: RefCell<Option<HandlerI2>>,
    on_rows_viewport_changed: RefCell<Option<HandlerI2>>,
    on_column_resized: RefCell<Option<HandlerStrF32>>,
    on_group_clicked: RefCell<Option<Handler0>>,
}

#[derive(Clone, Default)]
pub struct ProcessesPortStub {
    inner: Rc<ProcessesStubState>,
}

impl std::fmt::Debug for ProcessesPortStub {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProcessesPortStub").finish()
    }
}

impl ProcessesPortStub {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn all(&self) -> Vec<UiProcessesPortMsg> {
        self.inner.messages.borrow().clone()
    }

    pub fn set_selected_pid(&self, pid: i32) {
        self.inner.selected_pid.set(pid);
    }

    pub fn emit_select_process(&self, pid: i32, idx: i32) {
        self.inner
            .on_select_process
            .borrow()
            .as_ref()
            .expect("on_select_process not registered")(pid, idx);
    }

    pub fn emit_terminate(&self) {
        self.inner
            .on_terminate
            .borrow()
            .as_ref()
            .expect("on_terminate not registered")();
    }
}

impl UiProcessesPort for ProcessesPortStub {
    fn send(&self, msg: UiProcessesPortMsg) {
        self.inner.messages.borrow_mut().push(msg);
    }

    fn get_selected_pid(&self) -> i32 {
        self.inner.selected_pid.get()
    }
}

impl UiProcessesBindings for ProcessesPortStub {
    fn on_sort_by<F>(&self, handler: F)
    where
        F: Fn(SharedString) + 'static,
    {
        *self.inner.on_sort_by.borrow_mut() = Some(Box::new(handler));
    }

    fn on_toggle_expand_group<F>(&self, handler: F)
    where
        F: Fn(SharedString) + 'static,
    {
        *self.inner.on_toggle_expand_group.borrow_mut() = Some(Box::new(handler));
    }

    fn on_terminate<F>(&self, handler: F)
    where
        F: Fn() + 'static,
    {
        *self.inner.on_terminate.borrow_mut() = Some(Box::new(handler));
    }

    fn on_select_process<F>(&self, handler: F)
    where
        F: Fn(i32, i32) + 'static,
    {
        *self.inner.on_select_process.borrow_mut() = Some(Box::new(handler));
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

    fn on_group_clicked<F>(&self, handler: F)
    where
        F: Fn() + 'static,
    {
        *self.inner.on_group_clicked.borrow_mut() = Some(Box::new(handler));
    }
}
