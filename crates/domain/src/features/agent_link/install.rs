use app_contracts::features::agent_link::{AgentLinkState, StartInProcess};
use app_contracts::features::agents::AgentStateRequest;
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
fn agent_link(cx: &Install<AgentLinkDeps>) -> anyhow::Result<AgentLinkFeature> {
    let AgentLinkDeps { elevation } = *cx.params();
    let (link, addr) = cx
        .state::<AgentLinkState>()
        .driven_by(move |port| AgentLinkActor::new(port, elevation));
    addr.send(OfferInProcessLater);
    if (elevation.asked_at_start)() && (elevation.elevated)() {
        addr.send(StartInProcess);
    }

    addr.send(AgentStateRequest);

    Ok(AgentLinkFeature(link))
}
