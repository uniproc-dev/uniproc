pub mod actions;
pub mod actor;
pub mod backend;
pub mod connection;
pub mod decode;
pub mod linux_report;
pub mod process_events;
pub mod providers;
pub mod rpc;
pub mod settings;
pub mod windows_feed;
pub mod windows_report;

use guinea::prelude::*;
use tracing::info;

pub struct AgentsFeature;

impl AppFeature for AgentsFeature {
    type Exports = ();

    fn install(self, app: &mut FeatureBuilder) -> anyhow::Result<()> {
        info!("Agents feature installed");

        actions::install(app);
        providers::wsl::wsl_agent_feature(app)?;
        match providers::synthetic::requested() {
            Some(processes) => providers::synthetic::install(app, processes)?,
            None => providers::windows::windows_agent_feature(app)?,
        }

        Ok(())
    }
}
