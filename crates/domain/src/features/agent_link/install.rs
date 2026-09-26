use app_contracts::features::agent_link::AgentLinkState;
use app_contracts::features::agents::{
    AgentStateRequest, ScanTick, WindowsActionRequest, WindowsAgentRuntimeEvent,
};
use guinea::prelude::*;

use super::actor::{AgentLinkActor, OfferInProcessLater};
use super::in_process::{InProcessStart, start_embedded};

#[derive(Clone, Copy)]
pub struct AgentLinkParams {
    pub start_in_process: InProcessStart,
}

impl Default for AgentLinkParams {
    fn default() -> Self {
        Self {
            start_in_process: start_embedded,
        }
    }
}

feature! {
    pub AgentLinkFeature {
        exports { AgentLinkState }
    }
}

#[installs]
fn agent_link(cx: &FeatureInitContext, params: &AgentLinkParams) -> anyhow::Result<AgentLinkFeature> {
    let start_in_process = params.start_in_process;
    let (link, addr) = cx
        .state::<AgentLinkState>()
        .driven_by(move |port| AgentLinkActor::new(port, start_in_process));
    addr.subscribe_on::<WindowsAgentRuntimeEvent>(Bus::Global);
    addr.subscribe_on::<ScanTick>(Bus::Global);
    addr.subscribe_on::<WindowsActionRequest>(Bus::Global);
    addr.subscribe_on::<AgentStateRequest>(Bus::Global);
    addr.send(OfferInProcessLater);

    GlobalEventBus::publish(AgentStateRequest);

    Ok(AgentLinkFeature(link))
}
