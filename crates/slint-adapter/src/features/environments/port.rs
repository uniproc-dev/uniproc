use crate::features::environments::UiEnvironmentsAdapter;
use crate::{EnvironmentsFeatureGlobal, WslDistro};
use app_contracts::features::environments::{UiEnvironmentsPort, WslDistroDto};
use context::icons::{keys, Icons};
use macros::slint_port_adapter;
use slint::{ComponentHandle, ModelRc, VecModel};

#[slint_port_adapter(window = AppWindow)]
impl UiEnvironmentsPort for UiEnvironmentsAdapter {
    fn set_host_icon_by_key(&self, ui: &AppWindow, icon_key: &str) {
        let global = ui.global::<EnvironmentsFeatureGlobal>();
        global.set_host_icon(Icons::resolve(icon_key));
        global.set_host_icon_key(icon_key.into());
    }

    fn set_wsl_distros(&self, ui: &AppWindow, distros: Vec<WslDistroDto>) {
        let model = distros
            .into_iter()
            .map(|distro| WslDistro {
                name: distro.name.clone().into(),
                is_installed: distro.is_installed,
                is_running: distro.is_running,
                icon: Icons::get_key(match () {
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
}
