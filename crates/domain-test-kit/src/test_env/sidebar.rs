use app_contracts::features::sidebar::{UiSidebarBindings, UiSidebarPort, UiSidebarPortMsg};
use forsl_core::test_kit::Interaction;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Default)]
struct SidebarStubState {
    messages: RefCell<Vec<UiSidebarPortMsg>>,
    on_side_bar_width_changed: RefCell<Option<Box<dyn Fn(u64)>>>,
}

#[derive(Clone, Default)]
pub struct SidebarPortStub {
    inner: Rc<SidebarStubState>,
}

impl SidebarPortStub {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn all(&self) -> Vec<UiSidebarPortMsg> {
        self.inner.messages.borrow().clone()
    }

    pub fn last_width(&self) -> Option<u64> {
        self.inner.messages.borrow().iter().rev().find_map(|msg| match msg {
            UiSidebarPortMsg::SetSideBarWidth(width) => Some(*width),
            _ => None,
        })
    }

    pub fn emit_side_bar_width_changed(&self, width: u64) -> Interaction<()> {
        self.inner
            .on_side_bar_width_changed
            .borrow()
            .as_ref()
            .expect("on_side_bar_width_changed not registered")(width);
        Interaction::new(())
    }
}

impl UiSidebarPort for SidebarPortStub {
    fn send(&self, msg: UiSidebarPortMsg) {
        self.inner.messages.borrow_mut().push(msg);
    }
}

impl UiSidebarBindings for SidebarPortStub {
    fn on_side_bar_width_changed<F>(&self, handler: F)
    where
        F: Fn(u64) + 'static,
    {
        *self.inner.on_side_bar_width_changed.borrow_mut() = Some(Box::new(handler));
    }
}
