use crate::feature::FeatureComponent;
use app_core::actor::{Addr, ManagedActor, UiThreadToken};
use app_core::lifecycle_tracker::LifecycleTracker;

pub trait UiAutoWire<P>: ManagedActor {
    const IS_COMPLETE: bool;
    fn wire_full(addr: &Addr<Self>, port: &P);
    fn wire_partial(addr: &Addr<Self>, port: &P);
}

pub struct AddrBuilder<'a, L: LifecycleTracker> {
    token: UiThreadToken,
    tracker: &'a L,
}

#[must_use = "ManagedAddrBuilder does nothing unless you call .ui_bind() or .finish()"]
pub struct ManagedAddrBuilder<A: ManagedActor> {
    addr: Addr<A>,
}

impl<'a, L: LifecycleTracker> AddrBuilder<'a, L> {
    pub fn new(token: UiThreadToken, tracker: &'a L) -> Self {
        Self { token, tracker }
    }

    pub fn managed<A: ManagedActor>(&self, actor: A) -> ManagedAddrBuilder<A> {
        let addr = Addr::new_managed(actor, self.token.clone(), self.tracker);
        ManagedAddrBuilder { addr }
    }
}

impl<A: ManagedActor> ManagedAddrBuilder<A> {
    pub fn ui_bind<P>(self, port: &P) -> Addr<A>
    where
        A: UiAutoWire<P>,
    {
        let actor_name = std::any::type_name::<A>()
            .split("::")
            .last()
            .unwrap_or("Actor");

        if A::IS_COMPLETE {
            A::wire_full(&self.addr, port);
            tracing::debug!(actor = actor_name, "binding: COMPLETE");
        } else {
            A::wire_partial(&self.addr, port);
            tracing::warn!(
                actor = actor_name,
                "binding: PARTIAL (incomplete schema coverage)"
            );
        }

        self.addr
    }

    pub fn finish(self) -> Addr<A> {
        self.addr
    }
}

impl<A: ManagedActor> From<ManagedAddrBuilder<A>> for Addr<A> {
    fn from(builder: ManagedAddrBuilder<A>) -> Self {
        builder.addr
    }
}
