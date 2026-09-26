use app_contracts::features::agent_link::AgentLinkState;
use app_contracts::features::agents::{AgentStateRequest, WindowsAgentRuntimeEvent};
use guinea::prelude::*;

use super::actor::{AgentLinkActor, OfferNativeLater};
use super::native;

#[derive(Clone, Copy)]
pub struct AgentLinkParams {
    pub open_native: fn(),
}

impl Default for AgentLinkParams {
    fn default() -> Self {
        Self {
            open_native: native::open_task_manager,
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
    let open_native = params.open_native;
    let (link, addr) = cx
        .state::<AgentLinkState>()
        .driven_by(move |port| AgentLinkActor::new(port, open_native));
    addr.subscribe_on::<WindowsAgentRuntimeEvent>(Bus::Global);
    addr.send(OfferNativeLater);

    GlobalEventBus::publish(AgentStateRequest);

    Ok(AgentLinkFeature(link))
}
