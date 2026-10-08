use app_contracts::features::agent_link::{AgentLinkState, StartInProcess};
use app_contracts::features::agents::{AgentStateRequest, WindowsAgentRuntimeEvent};
use guinea::prelude::*;
use guinea_plugin_store::StoreAccess;

use super::actor::{AgentLinkActor, OfferInProcessLater};
use super::elevation::Elevation;
use super::in_process::{InProcessStart, start_local};
use crate::features::settings::settings::GeneralSettings;

#[derive(Clone, Copy)]
pub struct AgentLinkDeps {
    pub start_in_process: InProcessStart,
    pub elevation: Elevation,
}

impl Default for AgentLinkDeps {
    fn default() -> Self {
        Self {
            start_in_process: start_local,
            elevation: Elevation::default(),
        }
    }
}

feature! {
    pub AgentLinkFeature {
        exports { AgentLinkState }
    }
}

#[installs]
fn agent_link(cx: &FeatureInitContext, deps: &AgentLinkDeps) -> anyhow::Result<AgentLinkFeature> {
    let AgentLinkDeps {
        start_in_process,
        elevation,
    } = *deps;
    let update_interval_ms = cx.settings::<GeneralSettings>().update_interval_ms();
    let (link, addr) = cx
        .state::<AgentLinkState>()
        .driven_by(move |port| AgentLinkActor::new(port, start_in_process, elevation, update_interval_ms));
    addr.subscribe_on::<WindowsAgentRuntimeEvent>(Bus::Global);
    addr.subscribe_on::<AgentStateRequest>(Bus::Global);
    addr.send(OfferInProcessLater);
    if (elevation.asked_at_start)() && (elevation.elevated)() {
        addr.send(StartInProcess);
    }

    GlobalEventBus::publish(AgentStateRequest);

    Ok(AgentLinkFeature(link))
}
