use forsl::app::Window;
mod actor;
pub mod domain;

pub use actor::{Init, InstallAgent, WslEnvActor};

use app_contracts::features::environments::{UiEnvironmentsBindings, UiEnvironmentsPort};
use forsl::feature::{ContextActorExt, WindowFeature, WindowFeatureInitContext};
use macros::window_feature;

#[window_feature]
pub fn wsl_feature<TWindow, P>(
    ctx: &mut WindowFeatureInitContext<TWindow>,
    ui_port: P,
) -> anyhow::Result<()>
where
    TWindow: Window,
    P: UiEnvironmentsPort + UiEnvironmentsBindings + Clone + 'static,
{
    let addr = ctx
        .actor_builder(WslEnvActor::new(ui_port.clone()))
        .ui_bind(&ui_port);

    addr.send(Init);
    Ok(())
}
