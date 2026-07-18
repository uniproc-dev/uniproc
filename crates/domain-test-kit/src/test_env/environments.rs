use app_contracts::features::environments::{
    UiEnvironmentsBindings, UiEnvironmentsPort, UiEnvironmentsPortMsg,
};
use forsl_core::test_kit::Interaction;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Default)]
struct EnvironmentsStubState {
    messages: RefCell<Vec<UiEnvironmentsPortMsg>>,
    on_install_agent: RefCell<Option<Box<dyn Fn(String)>>>,
}

#[derive(Clone, Default)]
pub struct EnvironmentsPortStub {
    inner: Rc<EnvironmentsStubState>,
}

impl EnvironmentsPortStub {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn all(&self) -> Vec<UiEnvironmentsPortMsg> {
        self.inner.messages.borrow().clone()
    }

    pub fn emit_install_agent(&self, distro: String) -> Interaction<()> {
        self.inner
            .on_install_agent
            .borrow()
            .as_ref()
            .expect("on_install_agent not registered")(distro);
        Interaction::new(())
    }
}

impl UiEnvironmentsPort for EnvironmentsPortStub {
    fn send(&self, msg: UiEnvironmentsPortMsg) {
        self.inner.messages.borrow_mut().push(msg);
    }
}

impl UiEnvironmentsBindings for EnvironmentsPortStub {
    fn on_install_agent<F>(&self, handler: F)
    where
        F: Fn(String) + 'static,
    {
        *self.inner.on_install_agent.borrow_mut() = Some(Box::new(handler));
    }
}
