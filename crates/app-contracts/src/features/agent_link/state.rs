use guinea::prelude::*;

use super::messages::{AgentLinkMsg, InProcess};
use crate::features::agents::AgentConnectionState;

#[derive(Clone, PartialEq, Debug)]
pub struct AgentLinkState {
    pub windows: AgentConnectionState,
    pub ever_connected: bool,
    pub in_process_offered: bool,
    pub in_process: InProcess,
}

impl Default for AgentLinkState {
    fn default() -> Self {
        Self {
            windows: AgentConnectionState::Connecting,
            ever_connected: false,
            in_process_offered: false,
            in_process: InProcess::Off,
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
            AgentLinkMsg::OfferInProcess => self.in_process_offered = true,
            AgentLinkMsg::InProcess(in_process) => self.in_process = in_process,
        }
    }
}
