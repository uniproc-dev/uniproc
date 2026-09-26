pub mod actor;
pub mod backend;
pub mod connection;
pub mod decode;
pub mod providers;
pub mod rpc;
pub mod settings;

use std::time::Duration;

use app_contracts::features::agents::ScanTick;
use guinea::prelude::*;
use settings::AgentSettings;
use tracing::info;

const MIN_SCAN_INTERVAL_MS: u64 = 100;

pub struct AgentsFeature;

impl AppFeature for AgentsFeature {
    fn install(self, app: &mut FeatureBuilder) -> anyhow::Result<()> {
        info!("Agents feature installed");

        let settings = AgentSettings::new()?;
        let interval = settings.scan_interval_ms().get().max(MIN_SCAN_INTERVAL_MS);
        app.repeat(Duration::from_millis(interval), || {
            GlobalEventBus::publish(ScanTick);
        })
        .named("scan");

        cfg_if::cfg_if! {
            if #[cfg(target_os = "windows")] {
                providers::wsl::wsl_agent_feature(app)?;
                match providers::synthetic::requested() {
                    Some(processes) => providers::synthetic::install(app, processes)?,
                    None => providers::windows::windows_agent_feature(app)?,
                }
            } else {
                providers::linux::linux_agent_feature(app)?;
            }
        }

        Ok(())
    }
}
