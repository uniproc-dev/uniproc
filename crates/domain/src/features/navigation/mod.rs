pub mod actor;
mod model;
mod settings;
mod state;

use crate::features::navigation::actor::{NavigationActor, Push};
use crate::navigation_impl::settings::NavigationSettings;
use app_contracts::features::navigation::PAGE_ROUTES;
use app_contracts::features::navigation::UiNavigationBindings;
use forsl::app::Window;
use forsl::feature::{
    AppFeature, AppFeatureInitContext, ContextActorExt, ContextStoreExt, WindowFeature,
    WindowFeatureInitContext,
};
use forsl::navigation::{Route, RouteRegistry};
use forsl::uri::ContextlessAppUri;
use forsl_macros::{app_feature, window_feature};
use std::borrow::Cow;
use std::sync::Arc;

#[app_feature]
pub fn navigation_registry_feature(ctx: &mut AppFeatureInitContext) -> anyhow::Result<()> {
    let registry = Arc::new(RouteRegistry::new());
    registry.replace_routes(
        PAGE_ROUTES
            .iter()
            .map(|route| Route {
                uri: ContextlessAppUri::new(
                    Cow::from(route.segment),
                    route
                        .features
                        .iter()
                        .map(|&s| Cow::from(s))
                        .collect::<Vec<_>>(),
                ),
            })
            .collect(),
    );
    ctx.shared.insert_arc(registry);
    Ok(())
}

#[window_feature]
pub fn navigation_feature<TWindow, P>(
    ctx: &mut WindowFeatureInitContext<TWindow>,
    ui_port: P,
) -> anyhow::Result<()>
where
    TWindow: Window,
    P: UiNavigationBindings + Clone + 'static,
{
    let store = ctx.store();
    let settings = NavigationSettings::new_with(&store)?;

    let registry = ctx.shared.get::<RouteRegistry>().unwrap();
    let actor = NavigationActor::new(registry.clone(), ctx.window_id);

    let addr = ctx.actor_builder(actor).ui_bind(&ui_port);

    let initial_path = settings.default_route_segment().get();
    addr.send(Push(initial_path));

    Ok(())
}
