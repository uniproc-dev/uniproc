use crate::features::environments::UiEnvironmentsAdapter;
use crate::{EnvironmentsFeatureGlobal, EnvsLoading, WslDistro};
use app_contracts::features::environments::{UiEnvironmentsPort, UiEnvironmentsPortMsg};
use context::icons::{keys, resolve_image};
use macros::slint_port_adapter;
use slint::{ComponentHandle, ModelRc, VecModel};

#[slint_port_adapter(window = AppWindow)]
impl UiEnvironmentsPort for UiEnvironmentsAdapter {
    fn send(&self, ui: &AppWindow, msg: UiEnvironmentsPortMsg) {
        match msg {
            UiEnvironmentsPortMsg::SetHostIconByKey(icon_key) => {
                let global = ui.global::<EnvironmentsFeatureGlobal>();
                global.set_host_icon(resolve_image(icon_key.as_str()));
                global.set_host_icon_key(icon_key.into());
            }
            UiEnvironmentsPortMsg::SetWslDistros(distros) => {
                let model = distros
                    .into_iter()
                    .map(|distro| WslDistro {
                        name: distro.name.clone().into(),
                        is_installed: distro.is_installed,
                        is_running: distro.is_running,
                        icon: resolve_image(match () {
                            _ if distro.name.to_lowercase().contains("ubuntu") => keys::UBUNTU,
                            _ if distro.name.to_lowercase().contains("docker") => keys::DOCKER,
                            _ => keys::LINUX,
                        }),
                        icon_key: "".into(),
                        latency_ms: distro.latency_ms,
                    })
                    .collect::<Vec<_>>();

                ui.global::<EnvironmentsFeatureGlobal>()
                    .set_wsl_distros(ModelRc::new(VecModel::from(model)));
            }
            UiEnvironmentsPortMsg::SetHostName(name) => {
                ui.global::<EnvironmentsFeatureGlobal>()
                    .set_host_name(name.into());
            }
            UiEnvironmentsPortMsg::SetSelectedEnv(name) => {
                ui.global::<EnvironmentsFeatureGlobal>()
                    .set_selected_env(name.into());
            }
            UiEnvironmentsPortMsg::SetHasWsl(has_wsl) => {
                ui.global::<EnvironmentsFeatureGlobal>()
                    .set_has_wsl(has_wsl);
            }
            UiEnvironmentsPortMsg::SetWslIsLoading(loading) => {
                ui.global::<EnvsLoading>().set_wsl_is_loading(loading);
            }
            UiEnvironmentsPortMsg::SetWslDistrosIsLoading(loading) => {
                ui.global::<EnvsLoading>()
                    .set_wsl_distros_is_loading(loading);
            }
        }
    }
}
