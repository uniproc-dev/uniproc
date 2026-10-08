use app_contracts::features::agent_link::{AgentLinkState, StartInProcess};
use app_contracts::features::agents::{AgentStateRequest, WindowsAgentInProcess, WindowsAgentRuntimeEvent};
use guinea::prelude::*;

use super::actor::{AgentLinkActor, OfferInProcessLater};
use super::elevation::Elevation;

#[derive(Clone, Copy, Default)]
pub struct AgentLinkDeps {
    pub elevation: Elevation,
}

feature! {
    pub AgentLinkFeature {
        exports { AgentLinkState }
    }
}

#[installs]
fn agent_link(cx: &FeatureInitContext, deps: &AgentLinkDeps) -> anyhow::Result<AgentLinkFeature> {
    let AgentLinkDeps { elevation } = *deps;
    let (link, addr) = cx
        .state::<AgentLinkState>()
        .driven_by(move |port| AgentLinkActor::new(port, elevation));
    addr.subscribe_on::<WindowsAgentRuntimeEvent>(Bus::Global);
    addr.subscribe_on::<WindowsAgentInProcess>(Bus::Global);
    addr.send(OfferInProcessLater);
    if (elevation.asked_at_start)() && (elevation.elevated)() {
        addr.send(StartInProcess);
    }

    GlobalEventBus::publish(AgentStateRequest);

    Ok(AgentLinkFeature(link))
}
