use app_contracts::features::agents::RemoteScanResult;
use app_contracts::features::wsl::WslState;
use guinea::prelude::*;
use std::time::Duration;

use super::actor::{RefreshDistros, WslActor};
use super::scanner::{self, DistroScan};
use crate::features::agents::settings::AgentSettings;

const DISTRO_SCAN_INTERVAL: Duration = Duration::from_secs(3);

#[derive(Clone, Copy)]
pub struct WslDeps {
    pub scan: fn(Duration) -> DistroScan,
}

impl Default for WslDeps {
    fn default() -> Self {
        Self {
            scan: |timeout| Box::pin(scanner::scan_distros(timeout)),
        }
    }
}

feature! {
    pub WslFeature {
        exports { WslState }
    }
}

#[installs]
fn wsl(cx: &FeatureInitContext, deps: &WslDeps) -> anyhow::Result<WslFeature> {
    let settings = AgentSettings::new()?;
    let configured = settings.wsl_distro().get();
    let scan = deps.scan;

    let (wsl, addr) = cx
        .state::<WslState>()
        .driven_by(move |push| WslActor::new(push, configured, scan));

    cx.every(DISTRO_SCAN_INTERVAL, &addr, || RefreshDistros)
        .named("wsl-distro-scan");

    addr.subscribe_on::<RemoteScanResult>(Bus::Global);

    addr.send(RefreshDistros);

    Ok(WslFeature(wsl))
}
