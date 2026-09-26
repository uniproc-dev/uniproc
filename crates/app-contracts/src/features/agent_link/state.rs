use guinea::prelude::*;

use super::messages::AgentLinkMsg;
use crate::features::agents::AgentConnectionState;

#[derive(Clone, PartialEq, Debug)]
pub struct AgentLinkState {
    pub windows: AgentConnectionState,
    pub ever_connected: bool,
    pub native_offered: bool,
}

impl Default for AgentLinkState {
    fn default() -> Self {
        Self {
            windows: AgentConnectionState::Connecting,
            ever_connected: false,
            native_offered: false,
        }
    }
}

impl AgentLinkState {
    pub fn awaiting_first_connection(&self) -> bool {
        !self.ever_connected
    }
}

impl Reducer for AgentLinkState {
    type Update = AgentLinkMsg;

    fn reduce(&mut self, update: AgentLinkMsg) {
        match update {
            AgentLinkMsg::Windows(state) => {
                self.windows = state;
                self.ever_connected |= state == AgentConnectionState::Connected;
            }
            AgentLinkMsg::OfferNative => self.native_offered = true,
        }
    }
}
