use app_contracts::features::agent_link::AgentLinkState;
use app_contracts::features::agents::{AgentStateRequest, WindowsAgentRuntimeEvent};
use guinea::prelude::*;
use guinea_plugin_store::StoreAccess;

use super::actor::{AgentLinkActor, OfferInProcessLater};
use super::in_process::{InProcessStart, start_local};
use crate::features::settings::settings::GeneralSettings;

#[derive(Clone, Copy)]
pub struct AgentLinkDeps {
    pub start_in_process: InProcessStart,
}

impl Default for AgentLinkDeps {
    fn default() -> Self {
        Self {
            start_in_process: start_local,
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
    let start_in_process = deps.start_in_process;
    let update_interval_ms = cx.settings::<GeneralSettings>()?.update_interval_ms();
    let (link, addr) = cx
        .state::<AgentLinkState>()
        .driven_by(move |port| AgentLinkActor::new(port, start_in_process, update_interval_ms));
    addr.subscribe_on::<WindowsAgentRuntimeEvent>(Bus::Global);
    addr.subscribe_on::<AgentStateRequest>(Bus::Global);
    addr.send(OfferInProcessLater);

    GlobalEventBus::publish(AgentStateRequest);

    Ok(AgentLinkFeature(link))
}
