use crate::environments_impl::discovery::host::HostProviderActor;
use crate::environments_impl::discovery::wsl::WslDiscoveryActor;
use crate::environments_impl::registry::EnvironmentRegistryActor;
use crate::environments_impl::settings::EnvironmentsSettings;
use app_contracts::features::environments::{UiEnvironmentsBindings, UiEnvironmentsPort};
use app_core::actor::Addr;
use framework::app::Window;
use framework::feature::{
    AppFeature, AppFeatureInitContext, WindowFeature, WindowFeatureInitContext,
};
use macros::window_feature;

pub mod discovery;
mod registry;
mod settings;
pub mod wsl;

pub struct EnvironmentsRegistryFeature;

impl AppFeature for EnvironmentsRegistryFeature {
    fn install(&mut self, ctx: &mut AppFeatureInitContext) -> anyhow::Result<()> {
        let actor = EnvironmentRegistryActor::new();
        let settings = EnvironmentsSettings::new(ctx.shared)?;

        Addr::new_managed(actor, ctx.token.clone(), ctx.tracker);

        let host_addr = Addr::new_managed(HostProviderActor, ctx.token.clone(), ctx.tracker);
        host_addr.send(discovery::host::Init);

        #[cfg(windows)]
        {
            let wsl_addr = Addr::new_managed(WslDiscoveryActor, ctx.token.clone(), ctx.tracker);
            wsl_addr.send(discovery::wsl::Init);
            let h = ctx
                .reactor
                .add_dynamic_loop(settings.scan_interval_ms().as_signal(), move || {
                    wsl_addr.send(discovery::wsl::Refresh)
                });

            ctx.tracker.track_loop(h);
        }
        Ok(())
    }
}

#[window_feature]
pub struct EnvironmentsFeature;

#[window_feature]
impl<TWindow, F, P> WindowFeature<TWindow> for EnvironmentsFeature<F>
where
    TWindow: Window,
    F: Fn(&TWindow) -> P + Clone + 'static,
    P: UiEnvironmentsPort + UiEnvironmentsBindings + Clone + 'static,
{
    fn install(&mut self, ctx: &mut WindowFeatureInitContext<TWindow>) -> anyhow::Result<()> {
        wsl::WslFeature::new(self.make_port.clone()).install(ctx)?;
        Ok(())
    }
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
