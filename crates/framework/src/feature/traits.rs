use crate::app::Window;
use crate::feature::FeatureContextState;
use crate::lifecycle_tracker::{AppLifecycle, WindowLifecycle};
use crate::navigation::{RouteActivated, RouteDeactivated};
use crate::reactor::Reactor;
use crate::uri::AppUri;
use app_core::SharedState;
use app_core::actor::Addr;
use app_core::actor::Context;
use app_core::actor::UiThreadToken;
use app_core::actor::event_bus::EventBus;
use app_core::actor::event_bus::builder::EventSubscription;
use app_core::lifecycle_tracker::LifecycleTracker;
use app_core::trace::in_named_scope;
use std::marker::PhantomData;

pub struct WindowFeatureInitContext<'a, TWindow: Window> {
    pub window_id: usize,
    pub ui: &'a TWindow,
    pub shared: &'a SharedState,
    pub reactor: &'a Reactor,
    pub tracker: &'a WindowLifecycle<TWindow>,
    pub token: UiThreadToken,
}

impl<'a, TWindow: Window> WindowFeatureInitContext<'a, TWindow> {
    pub fn token(&self) -> UiThreadToken {
        self.ui.new_token()
    }
}

pub struct WindowFeatureDeinitContext<'a, TWindow: Window> {
    pub ui: &'a TWindow,
    pub shared: &'a SharedState,
    pub reactor: &'a Reactor,
    pub token: UiThreadToken,
}

pub struct AppFeatureInitContext<'a> {
    pub token: UiThreadToken,
    pub reactor: &'a Reactor,
    pub shared: &'a SharedState,
    pub tracker: &'a AppLifecycle,
}

pub struct AppFeatureDeinitContext<'a> {
    pub token: UiThreadToken,
    pub reactor: &'a Reactor,
    pub shared: &'a SharedState,
}

pub trait WindowFeature<TWindow: Window> {
    fn install(&mut self, ctx: &mut WindowFeatureInitContext<TWindow>) -> anyhow::Result<()>;
}

pub trait AppFeature {
    fn install(&mut self, ctx: &mut AppFeatureInitContext) -> anyhow::Result<()>;
}

pub trait FeatureComponent: Sized + 'static {
    fn context_state(&mut self) -> &mut FeatureContextState;
    fn on_activated(&mut self, uri: &AppUri, ctx: &Context<Self>);
    fn on_deactivated(&mut self, previous_uri: &AppUri, ctx: &Context<Self>);
}

pub trait FromMessage<Args> {
    fn from_msg(args: Args) -> Self;
}

pub struct Events<T>(PhantomData<T>);

impl<A, T> EventSubscription<A> for Events<T>
where
    A: FeatureComponent + 'static,
    T: EventSubscription<A>,
{
    fn subscribe_into(addr: Addr<A>, tracker: &impl LifecycleTracker) {
        let a_act = addr.clone();
        EventBus::subscribe_fn::<RouteActivated>(
            move |msg| {
                a_act.apply(move |actor, ctx| {
                    in_named_scope("framework.navigation.activate", None, None, || {
                        if let Some(key) = actor.context_state().handle_activation(&msg) {
                            actor.on_activated(key, ctx);
                        }
                    });
                });
            },
            tracker,
        );

        let a_deact = addr.clone();
        EventBus::subscribe_fn::<RouteDeactivated>(
            move |msg| {
                a_deact.apply(move |actor, ctx| {
                    in_named_scope("framework.navigation.deactivate", None, None, || {
                        if actor.context_state().handle_deactivation(&msg) {
                            actor.on_deactivated(&msg.uri, ctx);
                        }
                    });
                });
            },
            tracker,
        );

        T::subscribe_into(addr, tracker);
    }
}

pub trait IntoAppFeature {
    type Feature: AppFeature + 'static;
    fn into_feature(self) -> Self::Feature;
}

pub struct AppFeatureFn {
    f: fn(&mut AppFeatureInitContext) -> anyhow::Result<()>,
}

impl<T: AppFeature + 'static> IntoAppFeature for T {
    type Feature = T;
    fn into_feature(self) -> Self::Feature {
        self
    }
}

impl AppFeature for AppFeatureFn {
    fn install(&mut self, ctx: &mut AppFeatureInitContext) -> anyhow::Result<()> {
        (self.f)(ctx)
    }
}

impl IntoAppFeature for fn(&mut AppFeatureInitContext) -> anyhow::Result<()> {
    type Feature = AppFeatureFn;
    fn into_feature(self) -> Self::Feature {
        AppFeatureFn { f: self }
    }
}

pub trait FromWindow<TWindow> {
    fn from_window(ui: &TWindow) -> Self;
}

pub trait IntoWindowFeature<TWindow: Window> {
    type Feature: WindowFeature<TWindow> + 'static;
    fn into_feature(self) -> Self::Feature;
}

macro_rules! impl_window_feature_fn {
    ($($name:ident, $($port:ident),*);*) => {
        $(
            pub struct $name<TWindow: Window, $($port),*> {
                f: fn(&mut WindowFeatureInitContext<TWindow>, $($port),*) -> anyhow::Result<()>,
                _marker: PhantomData<(TWindow, $($port),*)>,
            }

            impl<TWindow, $($port),*> WindowFeature<TWindow> for $name<TWindow, $($port),*>
            where
                TWindow: Window,
                $($port: FromWindow<TWindow> + Clone + 'static),*
            {
                fn install(&mut self, ctx: &mut WindowFeatureInitContext<TWindow>) -> anyhow::Result<()> {
                    $(
                        let $port = <$port as FromWindow<TWindow>>::from_window(ctx.ui);
                    )*
                    (self.f)(ctx, $($port),*)
                }
            }

            impl<TWindow, $($port),*> IntoWindowFeature<TWindow> for fn(&mut WindowFeatureInitContext<TWindow>, $($port),*) -> anyhow::Result<()>
            where
                TWindow: Window,
                $($port: FromWindow<TWindow> + Clone + 'static),*
            {
                type Feature = $name<TWindow, $($port),*>;
                fn into_feature(self) -> Self::Feature {
                    $name {
                        f: self,
                        _marker: PhantomData,
                    }
                }
            }
        )*
    };
}

impl_window_feature_fn! {
    WindowFeatureFn0, ;
    WindowFeatureFn1, P1;
    WindowFeatureFn2, P1, P2;
    WindowFeatureFn3, P1, P2, P3;
    WindowFeatureFn4, P1, P2, P3, P4;
    WindowFeatureFn5, P1, P2, P3, P4, P5;
    WindowFeatureFn6, P1, P2, P3, P4, P5, P6;
    WindowFeatureFn7, P1, P2, P3, P4, P5, P6, P7;
    WindowFeatureFn8, P1, P2, P3, P4, P5, P6, P7, P8
}
