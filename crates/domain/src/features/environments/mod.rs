use crate::environments_impl::discovery::host::HostProviderActor;
use crate::environments_impl::discovery::wsl::WslDiscoveryActor;
use crate::environments_impl::registry::EnvironmentRegistryActor;
use crate::environments_impl::settings::EnvironmentsSettings;
use app_contracts::features::environments::{UiEnvironmentsBindings, UiEnvironmentsPort};
use forsl::app::Window;
use forsl::feature::{
    AppFeature, AppFeatureInitContext, ContextActorExt, ContextReactorExt, ContextStoreExt,
    WindowFeature, WindowFeatureInitContext,
};
use macros::{app_feature, window_feature};

pub mod discovery;
mod registry;
mod settings;
pub mod wsl;

#[app_feature]
pub fn environments_registry_feature(ctx: &mut AppFeatureInitContext) -> anyhow::Result<()> {
    let store = ctx.store();

    let actor = EnvironmentRegistryActor::new();
    let settings = EnvironmentsSettings::new(&store)?;

    ctx.spawn(actor);

    let host_addr = ctx.spawn(HostProviderActor);
    host_addr.send(discovery::host::Init);

    #[cfg(windows)]
    {
        let wsl_addr = ctx.spawn(WslDiscoveryActor);
        wsl_addr.send(discovery::wsl::Init);

        ctx.spawn_heartbeat(&wsl_addr, settings.scan_interval_ms().as_signal(), || {
            discovery::wsl::Refresh
        });
    }
    Ok(())
}

#[window_feature]
pub fn environments_feature<TWindow, P>(
    ctx: &mut WindowFeatureInitContext<TWindow>,
    ui_port: P,
) -> anyhow::Result<()>
where
    TWindow: Window,
    P: UiEnvironmentsPort + UiEnvironmentsBindings + Clone + 'static,
{
    wsl::wsl_feature(ctx, ui_port.clone())?;
    Ok(())
}

pub fn get_icon_for_env(name: &str) -> &'static str {
    let name_low = name.to_lowercase();

    match () {
        _ if name_low.contains("ubuntu") => "ubuntu",
        _ if name_low.contains("windows") => "windows-11",
        _ if name_low.contains("docker") => "docker",
        _ => "linux",
    }
}
