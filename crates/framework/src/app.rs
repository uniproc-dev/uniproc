use crate::feature::{
    AppFeature, AppFeatureDeinitContext, AppFeatureInitContext, WindowFeature,
    WindowFeatureDeinitContext, WindowFeatureInitContext,
};
use crate::lifecycle_tracker::FeatureLifecycle;
use crate::reactor::Reactor;
use app_core::SharedState;
use app_core::actor::{UiDispatcher, UiThreadToken};
use app_core::trace::in_named_scope;
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
        unsafe { UiThreadToken::new() }
    }
}

pub trait Window: ComponentHandle + UiContext + 'static {}
impl<TWindow: ComponentHandle + UiContext + 'static> Window for TWindow {}

pub struct App<TWindow> {
    ui: TWindow,
    reactor: Reactor,
    shared: SharedState,
    runtime: tokio::runtime::Runtime,
    window_factories: Vec<Box<dyn Fn() -> Box<dyn WindowFeature<TWindow>> + 'static>>,
    app_features: Vec<Box<dyn AppFeature>>,
    next_window_id: AtomicUsize,
    root_tracker: FeatureLifecycle,
}

impl<TWindow: Window> App<TWindow> {
    pub fn new(ui: TWindow) -> Self {
        Self::with_dispatcher(ui, SlintDispatcher)
    }

    pub fn with_dispatcher(ui: TWindow, dispatcher: impl UiDispatcher + 'static) -> Self {
        let runtime = tokio::runtime::Runtime::new().unwrap();

        let _guard = runtime.enter();

        dispatcher.init();

        Self {
            ui,
            runtime,
            reactor: Reactor::new(),
            shared: SharedState::new(),
            window_factories: Vec::new(),
            root_tracker: FeatureLifecycle::new(),
            app_features: Vec::new(),
            next_window_id: AtomicUsize::new(1),
        }
    }
    pub fn app_feature<F: AppFeature + 'static>(mut self, mut feature: F) -> anyhow::Result<Self> {
        let _guard = self.runtime.enter();

        let full_name = std::any::type_name::<F>();

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
            || match feature.install(&mut AppFeatureInitContext {
                token: self.ui.new_token(),
                reactor: &mut self.reactor,
                shared: &self.shared,
                tracker: &self.root_tracker,
            }) {
                Ok(_) => {
                    tracing::info!(
                        feature = clean_name,
                        status = "ok",
                        level = "app",
                        "feature.install"
                    );
                    self.app_features.push(Box::new(feature));
                    Ok(self)
                }
                Err(e) => {
                    tracing::error!(feature = clean_name, status = "error", level = "app", error = %e, "feature.install");
                    Err(e)
                }
            },
        )
    }

    pub fn window_feature<F, Builder>(mut self, builder: Builder) -> Self
    where
        Builder: Fn() -> F + 'static,
        F: WindowFeature<TWindow> + 'static,
    {
        self.window_factories
            .push(Box::new(move || Box::new(builder())));
        self
    }

    pub fn spawn_window(&mut self, ui: TWindow) -> anyhow::Result<()> {
        let _guard = self.runtime.enter();
        let window_id = self.next_window_id.fetch_add(1, Ordering::Relaxed);
        let mut active_features = Vec::new();

        for factory in &self.window_factories {
            let mut feature = factory();

            feature.install(&mut WindowFeatureInitContext {
                window_id,
                ui: &ui,
                shared: &self.shared,
                reactor: &mut self.reactor,
            })?;
            active_features.push(feature);
        }

        let features_storage = Rc::new(RefCell::new(active_features));
        let ui_clone = ui.clone_strong();

        ui.window().on_close_requested(move || {
            let features = std::mem::take(&mut *features_storage.borrow_mut());

            for feature in features {
                if let Err(e) = feature.uninstall(&mut WindowFeatureDeinitContext { ui: &ui_clone })
                {
                    tracing::error!("Error uninstalling feature: {}", e);
                }
            }

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

    pub fn run(mut self) -> anyhow::Result<()> {
        let _guard = self.runtime.enter();

        self.spawn_window(self.ui.clone_strong())?;

        let result = self.ui.run();

        tracing::info!("Application shutting down, uninstalling app features...");

        let token = self.ui.new_token();
        for feature in std::mem::take(&mut self.app_features) {
            let mut deinit_ctx = AppFeatureDeinitContext {
                token: token.clone(),
                reactor: &mut self.reactor,
                shared: &self.shared,
            };
            if let Err(e) = feature.uninstall(&mut deinit_ctx) {
                tracing::error!("Error during app feature uninstall: {}", e);
            }
        }

        result.map_err(|e| anyhow::anyhow!("UI execution error: {}", e))
    }
}
