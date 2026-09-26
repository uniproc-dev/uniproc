use app_contracts::features::agents::RemoteScanResult;
use app_contracts::features::wsl::WslState;
use guinea::prelude::*;
use std::time::Duration;

use super::actor::{RefreshDistros, WslActor};
use crate::features::agents::settings::AgentSettings;

const DISTRO_SCAN_INTERVAL: Duration = Duration::from_secs(3);

feature! {
    pub WslFeature {
        exports { WslState }
    }
}

#[installs]
fn wsl(cx: &FeatureInitContext) -> anyhow::Result<WslFeature> {
    let settings = AgentSettings::new()?;
    let configured = settings.wsl_distro().get();

    let (wsl, addr) = cx
        .state::<WslState>()
        .driven_by(|push| WslActor::new(push, configured));

    cx.every(DISTRO_SCAN_INTERVAL, &addr, || RefreshDistros)
        .named("wsl-distro-scan");

    addr.subscribe_on::<RemoteScanResult>(Bus::Global);

    addr.send(RefreshDistros);

    Ok(WslFeature(wsl))
}
