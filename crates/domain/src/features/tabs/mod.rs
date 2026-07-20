pub mod actor;
mod model;
mod state;

use crate::features::tabs::actor::TabsActor;
use crate::features::tabs::model::bootstrap_contexts;
use app_contracts::features::tabs::{UiTabsBindings, UiTabsPort, UiTabsPortMsg};
use forsl::app::Window;
use forsl::feature::{ContextActorExt, WindowFeature, WindowFeatureInitContext};
use forsl::navigation::RouteRegistry;
use forsl_macros::window_feature;

#[window_feature]
pub fn tabs_feature<TWindow, P>(
    ctx: &mut WindowFeatureInitContext<TWindow>,
    ui_port: P,
) -> anyhow::Result<()>
where
    TWindow: Window,
    P: UiTabsPort + UiTabsBindings + Clone + 'static,
{
    let routes = ctx
        .shared
        .get::<RouteRegistry>()
        .expect("RouteRegistry must be installed before TabsFeature");

    let contexts = bootstrap_contexts();
    let actor = TabsActor::new(ui_port.clone(), contexts, routes);

    let tabs = actor.tabs().to_vec();
    let available_contexts = actor.available_contexts().to_vec();
    let active_context_key = actor.active_context_key().cloned();

    ctx.actor_builder(actor).ui_bind(&ui_port);

    ui_port.send(UiTabsPortMsg::SetTabs(tabs));
    ui_port.send(UiTabsPortMsg::SetAvailableContexts(available_contexts));
    if let Some(active_context_key) = active_context_key {
        ui_port.send(UiTabsPortMsg::SetActiveContext(active_context_key));
    }

    Ok(())
}
