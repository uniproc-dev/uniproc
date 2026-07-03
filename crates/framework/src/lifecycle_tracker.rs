use crate::app::Window;
use crate::feature::{AppFeatureDeinitContext, WindowFeatureDeinitContext};
use app_core::actor::addr::Addr;
use app_core::actor::event_bus::subscribe::SubscriptionId;
use app_core::actor::event_bus::EventBus;
use app_core::actor::UiThreadToken;
use app_core::lifecycle_tracker::LifecycleTracker;
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Default)]
struct LifecycleCore {
    subs: Vec<SubscriptionId>,
    actor_counters: Vec<Rc<&'static str>>,
    anchors: Vec<Box<dyn Any>>,
}

impl LifecycleCore {
    fn track_loop<T: 'static>(&mut self, handle: T) {
        self.anchors.push(Box::new(handle));
    }

    fn track_actor<A: 'static>(&mut self, addr: &Addr<A>) {
        self.actor_counters.push(addr.strong_count_ptr());
    }

    fn track_sub(&mut self, id: SubscriptionId) {
        self.subs.push(id);
    }

    fn shutdown(&mut self, token: &UiThreadToken) {
        for sub_id in self.subs.drain(..) {
            EventBus::unsubscribe(token, sub_id);
        }
        let counters = std::mem::take(&mut self.actor_counters);
        self.anchors.clear();
        for counter in counters {
            let count = Rc::strong_count(&counter);
            if count > 1 {
                tracing::error!(
                    "LEAK: Actor<{}> still alive (refs: {})",
                    *counter,
                    count - 1
                );
            }
        }
    }
}

// --- AppLifecycle ---

#[derive(Clone, Default)]
pub struct AppLifecycle {
    inner: Rc<RefCell<AppLifecycleInner>>,
}

#[derive(Default)]
struct AppLifecycleInner {
    core: LifecycleCore,
    cleanups: Vec<Box<dyn for<'a> FnOnce(&mut AppFeatureDeinitContext<'a>) -> anyhow::Result<()>>>,
}

impl AppLifecycle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn on_cleanup(
        &self,
        f: impl for<'a> FnOnce(&mut AppFeatureDeinitContext<'a>) -> anyhow::Result<()> + 'static,
    ) {
        self.inner.borrow_mut().cleanups.push(Box::new(f));
    }

    pub fn shutdown(self, token: &UiThreadToken, ctx: &mut AppFeatureDeinitContext<'_>) {
        let mut inner = self.inner.borrow_mut();
        for cleanup in inner.cleanups.drain(..).rev() {
            if let Err(e) = cleanup(ctx) {
                tracing::error!("App cleanup error: {}", e);
            }
        }
        inner.core.shutdown(token);
    }

    pub fn track_loop<T: 'static>(&self, handle: T) {
        self.inner.borrow_mut().core.track_loop(handle);
    }

    pub fn track_actor<A: 'static>(&self, addr: &Addr<A>) {
        self.inner.borrow_mut().core.track_actor(addr);
    }

    pub fn track_sub(&self, id: SubscriptionId) {
        self.inner.borrow_mut().core.track_sub(id);
    }
}

impl LifecycleTracker for AppLifecycle {
    fn track_loop<T: 'static>(&self, handle: T) {
        self.track_loop(handle);
    }
    fn track_actor<A: 'static>(&self, addr: &Addr<A>) {
        self.track_actor(addr);
    }
    fn track_sub(&self, id: SubscriptionId) {
        self.track_sub(id);
    }
}

pub struct WindowLifecycle<TWindow: Window> {
    inner: Rc<RefCell<WindowLifecycleInner<TWindow>>>,
}

impl<TWindow: Window> Clone for WindowLifecycle<TWindow> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

struct WindowLifecycleInner<TWindow: Window> {
    core: LifecycleCore,
    cleanups: Vec<
        Box<dyn for<'a> FnOnce(&mut WindowFeatureDeinitContext<'a, TWindow>) -> anyhow::Result<()>>,
    >,
}

impl<TWindow: Window> Default for WindowLifecycleInner<TWindow> {
    fn default() -> Self {
        Self {
            core: LifecycleCore::default(),
            cleanups: Vec::new(),
        }
    }
}

impl<TWindow: Window> WindowLifecycle<TWindow> {
    pub fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(WindowLifecycleInner::default())),
        }
    }

    pub fn on_cleanup(
        &self,
        f: impl for<'a> FnOnce(&mut WindowFeatureDeinitContext<'a, TWindow>) -> anyhow::Result<()>
        + 'static,
    ) {
        self.inner.borrow_mut().cleanups.push(Box::new(f));
    }

    pub fn shutdown(
        self,
        token: &UiThreadToken,
        ctx: &mut WindowFeatureDeinitContext<'_, TWindow>,
    ) {
        let mut inner = self.inner.borrow_mut();
        for cleanup in inner.cleanups.drain(..).rev() {
            if let Err(e) = cleanup(ctx) {
                tracing::error!("Window cleanup error: {}", e);
            }
        }
        inner.core.shutdown(token);
    }

    pub fn track_loop<T: 'static>(&self, handle: T) {
        self.inner.borrow_mut().core.track_loop(handle);
    }

    pub fn track_actor<A: 'static>(&self, addr: &Addr<A>) {
        self.inner.borrow_mut().core.track_actor(addr);
    }

    pub fn track_sub(&self, id: SubscriptionId) {
        self.inner.borrow_mut().core.track_sub(id);
    }
}

impl<TWindow: Window> LifecycleTracker for WindowLifecycle<TWindow> {
    fn track_loop<T: 'static>(&self, handle: T) {
        self.track_loop(handle);
    }
    fn track_actor<A: 'static>(&self, addr: &Addr<A>) {
        self.track_actor(addr);
    }
    fn track_sub(&self, id: SubscriptionId) {
        self.track_sub(id);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::AppFeatureDeinitContext;
    use crate::reactor::Reactor;
    use app_core::actor::UiThreadToken;
    use app_core::SharedState;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    struct DropCheck(Arc<AtomicUsize>);
    impl Drop for DropCheck {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[test]
    fn test_lifecycle_anchors_cleanup() {
        let lifecycle = AppLifecycle::new();
        let counter = Arc::new(AtomicUsize::new(0));

        lifecycle.track_loop(DropCheck(counter.clone()));
        assert_eq!(counter.load(Ordering::SeqCst), 0);

        let token = UiThreadToken::dangerously_create_token_unchecked();
        let reactor = Reactor::new();
        let shared = SharedState::new();
        let mut ctx = AppFeatureDeinitContext {
            token: token.clone(),
            reactor: &reactor,
            shared: &shared,
        };

        lifecycle.shutdown(&token, &mut ctx);

        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_lifecycle_subs_drain() {
        let lifecycle = AppLifecycle::new();
        lifecycle.track_sub(1);
        lifecycle.track_sub(2);

        {
            let inner = lifecycle.inner.borrow();
            assert_eq!(inner.core.subs.len(), 2);
        }

        let token = UiThreadToken::dangerously_create_token_unchecked();
        let reactor = Reactor::new();
        let shared = SharedState::new();
        let mut ctx = AppFeatureDeinitContext {
            token: token.clone(),
            reactor: &reactor,
            shared: &shared,
        };

        lifecycle.clone().shutdown(&token, &mut ctx);

        {
            let inner = lifecycle.inner.borrow();
            assert_eq!(inner.core.subs.len(), 0);
        }
    }
}
