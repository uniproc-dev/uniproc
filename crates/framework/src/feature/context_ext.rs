use crate::addr::{AddrBuilder, UiAutoWire};
use crate::app::Window;
use crate::feature::{AppFeatureInitContext, WindowFeatureInitContext};
use crate::into_signal::IntoSignal;
use crate::lifecycle_tracker::{AppLifecycle, WindowLifecycle};
use crate::reactor::{LoopHandle, Reactor};
use app_core::actor::{Addr, Handler, ManagedActor, Message, UiThreadToken};
use app_core::lifecycle_tracker::LifecycleTracker;
use app_core::signal::Signal;
use app_core::SharedState;
use rpstate::DefaultStore;
use std::sync::Arc;

pub trait FeatureContext {
    type Tracker: LifecycleTracker;
    fn token(&self) -> UiThreadToken;
    fn tracker(&self) -> &Self::Tracker;
    fn reactor(&self) -> &Reactor;
    fn shared(&self) -> &SharedState;
}

impl<TWindow: Window> FeatureContext for WindowFeatureInitContext<'_, TWindow> {
    type Tracker = WindowLifecycle<TWindow>;
    fn token(&self) -> UiThreadToken {
        self.ui.new_token()
    }
    fn tracker(&self) -> &Self::Tracker {
        self.tracker
    }
    fn reactor(&self) -> &Reactor {
        self.reactor
    }
    fn shared(&self) -> &SharedState {
        self.shared
    }
}

impl FeatureContext for AppFeatureInitContext<'_> {
    type Tracker = AppLifecycle;
    fn token(&self) -> UiThreadToken {
        self.token.clone()
    }
    fn tracker(&self) -> &Self::Tracker {
        self.tracker
    }
    fn reactor(&self) -> &Reactor {
        self.reactor
    }
    fn shared(&self) -> &SharedState {
        self.shared
    }
}

pub struct ActorBuilder<'a, Ctx: FeatureContext, A: ManagedActor> {
    ctx: &'a mut Ctx,
    actor: A,
}

impl<'a, Ctx: FeatureContext, A: ManagedActor> ActorBuilder<'a, Ctx, A> {
    pub fn ui_bind<P>(self, port: &P) -> Addr<A>
    where
        A: UiAutoWire<P>,
    {
        let addr = AddrBuilder::new(self.ctx.token(), self.ctx.tracker())
            .managed(self.actor)
            .ui_bind(port);
        post_spawn(self.ctx, &addr);
        addr
    }

    pub fn build(self) -> Addr<A> {
        let addr = Addr::new_managed(self.actor, self.ctx.token(), self.ctx.tracker());
        self.ctx.tracker().track_actor(&addr);
        post_spawn(self.ctx, &addr);
        addr
    }
}

fn post_spawn<Ctx: FeatureContext, A: ManagedActor>(ctx: &mut Ctx, addr: &Addr<A>) {
    //TODO remove flag for introspection
    #[cfg(feature = "test-utils")]
    if let Some(reg) = ctx
        .shared()
        .get::<app_core::actor::registry::ActorRegistry>()
    {
        reg.register(addr.clone());
    }
}

pub trait ContextActorExt: FeatureContext + Sized {
    fn spawn<A: ManagedActor>(&mut self, actor: A) -> Addr<A> {
        self.actor_builder(actor).build()
    }

    fn actor_builder<A: ManagedActor>(&mut self, actor: A) -> ActorBuilder<'_, Self, A> {
        ActorBuilder { ctx: self, actor }
    }
}

impl<Ctx: FeatureContext> ContextActorExt for Ctx {}

pub trait ContextReactorExt: FeatureContext {
    fn spawn_periodic_send<A, M>(
        &mut self,
        addr: &Addr<A>,
        interval: impl IntoSignal<u64>,
        active: impl IntoSignal<bool>,
        msg_factory: impl Fn() -> M + Send + 'static,
    ) where
        A: Handler<M>,
        M: Message + Send + 'static,
    {
        let addr = addr.clone();
        let handle: LoopHandle = self.reactor().add_loop(interval, active, move || {
            addr.send(msg_factory());
        });
        self.tracker().track_loop(handle);
    }

    fn spawn_heartbeat<A, M>(
        &mut self,
        addr: &Addr<A>,
        interval: impl IntoSignal<u64>,
        msg_factory: impl Fn() -> M + Send + 'static,
    ) where
        A: Handler<M>,
        M: Message + Send + 'static,
    {
        self.spawn_periodic_send(addr, interval, Signal::new(true), msg_factory);
    }
}

impl<Ctx: FeatureContext> ContextReactorExt for Ctx {}

pub trait ContextStoreExt: FeatureContext {
    fn store(&self) -> Arc<DefaultStore> {
        self.shared()
            .get::<DefaultStore>()
            .expect("DefaultStore must be registered in SharedState")
    }
}

impl<Ctx: FeatureContext> ContextStoreExt for Ctx {}
