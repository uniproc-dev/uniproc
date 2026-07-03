use framework::app::Window;
mod actor;

use crate::features::window_actions::actor::WindowActor;
use app_contracts::features::window_actions::{UiWindowActionsBindings, UiWindowActionsPort};
use framework::feature::{ContextActorExt, WindowFeature, WindowFeatureInitContext};
use macros::window_feature;

#[window_feature]
pub fn window_actions_feature<TWindow, P>(
    ctx: &mut WindowFeatureInitContext<TWindow>,
    port: P,
) -> anyhow::Result<()>
where
    TWindow: Window,
    P: UiWindowActionsPort + UiWindowActionsBindings + Clone + 'static,
{
    ctx.actor_builder(WindowActor { port: port.clone() })
        .ui_bind(&port);
    Ok(())
}
