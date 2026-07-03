use crate::feature::{
    AppFeature, AppFeatureDeinitContext, AppFeatureInitContext, IntoAppFeature, IntoWindowFeature,
    WindowFeature, WindowFeatureDeinitContext, WindowFeatureInitContext,
};
use crate::lifecycle_tracker::{AppLifecycle, WindowLifecycle};
use crate::reactor::Reactor;
use app_core::actor::{UiDispatcher, UiThreadToken};
use app_core::trace::in_named_scope;
use app_core::SharedState;
use slint::ComponentHandle;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

struct SlintDispatcher;

impl UiDispatcher for SlintDispatcher {
    fn init(&self) {
        app_core::actor::set_ui_dispatcher(SlintDispatcher);
    }

    fn dispatch(&self, task: app_core::actor::UiTask) {
        let _ = slint::invoke_from_event_loop(task);
    }
}

pub trait UiContext {
    fn new_token(&self) -> UiThreadToken;
}

impl<TWindow: ComponentHandle + 'static> UiContext for TWindow {
    fn new_token(&self) -> UiThreadToken {
        UiThreadToken::dangerously_create_token_unchecked()
    }
}

pub trait Window: ComponentHandle + UiContext + 'static {}
impl<TWindow: ComponentHandle + UiContext + 'static> Window for TWindow {}

pub struct App<TWindow> {
    ui: TWindow,
    shared: SharedState,
    runtime: tokio::runtime::Runtime,
    next_window_id: Rc<AtomicUsize>,
    inner: Rc<RefCell<AppInner<TWindow>>>,
}

type BoxedWindowFeature<TWindow> = Box<dyn Fn() -> Box<dyn WindowFeature<TWindow>> + 'static>;

struct AppInner<TWindow> {
    reactor: Reactor,
    window_factories: Vec<BoxedWindowFeature<TWindow>>,
    app_features: Vec<Box<dyn AppFeature>>,
    root_tracker: AppLifecycle,
}

impl<TWindow: Window> App<TWindow> {
    pub fn new(ui: TWindow) -> anyhow::Result<Self> {
        Self::with_dispatcher(ui, SlintDispatcher)
    }

    pub fn with_dispatcher(
        ui: TWindow,
        dispatcher: impl UiDispatcher + 'static,
    ) -> anyhow::Result<Self> {
        let runtime = tokio::runtime::Runtime::new()?;
        let _guard = runtime.enter();

        dispatcher.init();

        let shared = SharedState::new();

        Ok(Self {
            ui,
            shared,
            runtime,
            next_window_id: Rc::new(AtomicUsize::new(1)),
            inner: Rc::new(RefCell::new(AppInner {
                reactor: Reactor::new(),
                window_factories: Vec::new(),
                app_features: Vec::new(),
                root_tracker: AppLifecycle::new(),
            })),
        })
    }

    pub fn app_feature<I: IntoAppFeature + 'static>(
        self,
        mut into_feature: I,
    ) -> anyhow::Result<Self> {
        let _guard = self.runtime.enter();

        let full_name = std::any::type_name::<I>();
        let clean_name = full_name
            .split('<')
            .next()
            .unwrap_or(full_name)
            .split("::")
            .last()
            .unwrap_or("Unknown");

        in_named_scope(
            "core.app.feature_install",
            Some("feature,status,level"),
            Some(format!("{}|ok|app", clean_name)),
            || {
                let mut inner = self.inner.borrow_mut();
                let mut init_ctx = AppFeatureInitContext {
                    token: self.ui.new_token(),
                    reactor: &inner.reactor,
                    shared: &self.shared,
                    tracker: &inner.root_tracker,
                };

                let mut feature = into_feature.into_feature();

                match feature.install(&mut init_ctx) {
                    Ok(_) => {
                        tracing::info!(
                            feature = clean_name,
                            status = "ok",
                            level = "app",
                            "feature.install"
                        );
                        inner.app_features.push(Box::new(feature));
                        drop(inner);
                        Ok(self)
                    }
                    Err(e) => {
                        tracing::error!(
                            feature = clean_name,
                            status = "error",
                            level = "app",
                            error = %e,
                            "feature.install"
                        );
                        Err(e)
                    }
                }
            },
        )
    }

    pub fn window_feature<I>(self, into_feature: I) -> Self
    where
        I: IntoWindowFeature<TWindow> + Clone + 'static,
    {
        self.inner
            .borrow_mut()
            .window_factories
            .push(Box::new(move || {
                Box::new(into_feature.clone().into_feature())
            }));
        self
    }

    pub fn spawn_window(&self, ui: TWindow) -> anyhow::Result<()> {
        let _guard = self.runtime.enter();
        let window_id = self.next_window_id.fetch_add(1, Ordering::Relaxed);
        let token = ui.new_token();
        let window_tracker = WindowLifecycle::new();
        let mut active_features = Vec::new();

        let inner = self.inner.borrow();

        for factory in &inner.window_factories {
            let mut feature = factory();
            let mut init_ctx = WindowFeatureInitContext {
                window_id,
                ui: &ui,
                shared: &self.shared,
                reactor: &inner.reactor,
                tracker: &window_tracker,
                token: token.clone(),
            };
            feature.install(&mut init_ctx)?;
            active_features.push(feature);
        }

        let features_storage = Rc::new(RefCell::new(active_features));
        let tracker = window_tracker;
        let token_for_close = token;
        let ui_clone = ui.clone_strong();

        let inner_clone = Rc::clone(&self.inner);
        let shared_clone = self.shared.clone();

        ui.window().on_close_requested(move || {
            let inner_borrow = inner_clone.borrow();

            let mut deinit_ctx = WindowFeatureDeinitContext {
                ui: &ui_clone,
                token: token_for_close.clone(),
                reactor: &inner_borrow.reactor,
                shared: &shared_clone,
            };

            tracker.clone().shutdown(&token_for_close, &mut deinit_ctx);

            let _ = std::mem::take(&mut *features_storage.borrow_mut());

            slint::CloseRequestResponse::HideWindow
        });

        Ok(())
    }

    pub fn ui(&self) -> &TWindow {
        &self.ui
    }

    pub fn shared(&self) -> &SharedState {
        &self.shared
    }

    pub fn run(self) -> anyhow::Result<()> {
        let _guard = self.runtime.enter();

        self.spawn_window(self.ui.clone_strong())?;

        let result = self.ui.run();

        tracing::info!("Application shutting down, executing app feature cleanups...");

        let inner = self.inner.borrow();
        let token = self.ui.new_token();

        let mut deinit_ctx = AppFeatureDeinitContext {
            token: token.clone(),
            reactor: &inner.reactor,
            shared: &self.shared,
        };

        inner.root_tracker.clone().shutdown(&token, &mut deinit_ctx);

        drop(inner);

        result.map_err(|e| anyhow::anyhow!("UI execution error: {}", e))
    }
}
