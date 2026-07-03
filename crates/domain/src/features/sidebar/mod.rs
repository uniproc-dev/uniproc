pub mod actor;
pub mod settings;

use crate::features::sidebar::actor::SidebarActor;
use crate::features::sidebar::settings::SidebarSettings;
use app_contracts::features::sidebar::{UiSidebarBindings, UiSidebarPort};
use framework::app::Window;
use framework::feature::{
    ContextActorExt, ContextStoreExt, WindowFeature, WindowFeatureInitContext,
};
use macros::window_feature;

#[window_feature]
pub fn sidebar_feature<TWindow, P>(
    ctx: &mut WindowFeatureInitContext<TWindow>,
    ui_port: P,
) -> anyhow::Result<()>
where
    TWindow: Window,
    P: UiSidebarPort + UiSidebarBindings + Clone + 'static,
{
    let store = ctx.store();

    let settings = SidebarSettings::new(&store)?;
    ui_port.set_side_bar_width(settings.width().get());

    let actor = SidebarActor::new(ui_port.clone(), settings);

    ctx.actor_builder(actor).ui_bind(&ui_port);

    Ok(())
}
