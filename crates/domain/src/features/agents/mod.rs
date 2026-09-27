pub mod actor;
pub mod backend;
pub mod connection;
pub mod decode;
pub mod providers;
pub mod rpc;
pub mod settings;
pub mod windows_report;

use app_contracts::features::agents::ScanTick;
use app_contracts::features::settings::{UpdateInterval, UpdateIntervalChanged};
use guinea::prelude::*;
use tracing::info;

use crate::features::settings::settings::GeneralSettings;

pub struct AgentsFeature;

impl AppFeature for AgentsFeature {
    fn install(self, app: &mut FeatureBuilder) -> anyhow::Result<()> {
        info!("Agents feature installed");

        let interval = GeneralSettings::new()?.update_interval();
        let scan = app
            .repeat(interval, || {
                GlobalEventBus::publish(ScanTick);
            })
            .named("scan");
        app.subscribe_global(move |UpdateIntervalChanged(ms)| {
            scan.clone().period(UpdateInterval::clamp(ms));
        });

        providers::wsl::wsl_agent_feature(app)?;
        match providers::synthetic::requested() {
            Some(processes) => providers::synthetic::install(app, processes)?,
            None => providers::windows::windows_agent_feature(app)?,
        }

        Ok(())
    }
}
