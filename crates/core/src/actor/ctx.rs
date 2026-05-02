use crate::actor::addr::{Addr, REGISTRY};
use crate::actor::event_bus::subscribe::Event;
use crate::actor::event_bus::EventBus;
use crate::actor::traits::{Handler, Message};
use crate::actor::{invoke_on_ui, short_type_name, AllowedSignal, ManagedActor};
use crate::trace::{current_meta, install_current_meta, DispatchMeta};
use std::marker::PhantomData;
use tokio::sync::oneshot;

pub struct Context<A: 'static> {
    pub(super) addr: Addr<A>,
}

impl<A: 'static> Context<A> {
    pub fn addr(&self) -> Addr<A> {
        self.addr.clone()
    }

    pub fn publish<M>(&self, msg: M)
    where
        A: ManagedActor,
        M: Event,
        A::Signals: AllowedSignal<M>,
    {
        EventBus::publish(msg);
    }

    pub fn spawn_bg<M, Fut>(&self, fut: Fut)
    where
        M: Message + 'static + Send,
        A: Handler<M>,
        Fut: Future<Output = M> + 'static + Send,
    {
        let id = self.addr.id;
        let meta = current_meta().unwrap_or_else(|| DispatchMeta::capture_or_root("core.actor.bg"));
        let span = tracing::debug_span!(
            parent: &meta.span,
            "actor.bg",
            actor = short_type_name::<A>(),
            result = short_type_name::<M>(),
            op_id = meta.op_id,
            correlation_id = meta.correlation_id.as_deref().unwrap_or(""),
        );

        #[cfg(feature = "test-utils")]
        use crate::actor::event_bus::ACTIVE_TASKS;

        #[cfg(feature = "test-utils")]
        ACTIVE_TASKS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        tokio::spawn(async move {
            let _meta_guard = install_current_meta(meta.clone());
            let result = {
                let _enter = span.enter();
                fut.await
            };

            let return_task = move || {
                REGISTRY.with(|reg| {
                    if let Some(boxed_addr) = reg.borrow().get(&id) {
                        if let Some(addr) = boxed_addr.downcast_ref::<Addr<A>>() {
                            addr.send_with_meta(
                                result,
                                meta.child("core.actor.bg.result", None, None),
                            );
                        }
                    }

                    #[cfg(feature = "test-utils")]
                    ACTIVE_TASKS.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
                });
            };

            invoke_on_ui(return_task);
        });
    }
}

pub struct AsyncContext<A: 'static> {
    actor_id: usize,
    _phantom: PhantomData<A>,
}

impl<A: 'static> Clone for AsyncContext<A> {
    fn clone(&self) -> Self {
        Self {
            actor_id: self.actor_id,
            _phantom: PhantomData,
        }
    }
}

unsafe impl<A: 'static> Send for AsyncContext<A> {}
unsafe impl<A: 'static> Sync for AsyncContext<A> {}

impl<A: 'static> AsyncContext<A> {
    pub(crate) fn new(actor_id: usize) -> Self {
        Self {
            actor_id,
            _phantom: PhantomData,
        }
    }

    pub fn publish<M>(&self, msg: M)
    where
        A: ManagedActor,
        M: Event,
        A::Signals: AllowedSignal<M>,
    {
        EventBus::publish(msg);
    }

    pub async fn apply<R, F>(&self, f: F) -> R
    where
        F: FnOnce(&mut A, &Context<A>) -> R + Send + 'static,
        R: Send + 'static,
    {
        let (tx, rx) = oneshot::channel();
        let id = self.actor_id;

        invoke_on_ui(move || {
            REGISTRY.with(|reg| {
                let reg_borrow = reg.borrow();
                if let Some(boxed_addr) = reg_borrow.get(&id) {
                    if let Some(addr) = boxed_addr.downcast_ref::<Addr<A>>() {
                        addr.apply(move |actor, ctx| {
                            let result = f(actor, ctx);
                            let _ = tx.send(result);
                        });
                    }
                }
            });
        });

        rx.await
            .expect("Actor target dropped or UI thread panicked")
    }

    pub fn send<M>(&self, msg: M)
    where
        M: Message + Send,
        A: Handler<M>,
    {
        let id = self.actor_id;
        let meta = current_meta()
            .unwrap_or_else(|| DispatchMeta::capture_or_root("core.actor.async.send"));

        invoke_on_ui(move || {
            REGISTRY.with(|reg| {
                if let Some(addr) = reg
                    .borrow()
                    .get(&id)
                    .and_then(|a| a.downcast_ref::<Addr<A>>())
                {
                    addr.send_with_meta(msg, meta);
                }
            });
        });
    }
}

impl<A: 'static> Context<A> {
    pub fn async_ctx(&self) -> AsyncContext<A> {
        AsyncContext::new(self.addr.id)
    }
}
