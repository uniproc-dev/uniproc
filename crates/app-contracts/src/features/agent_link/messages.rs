use serde::Deserialize;

use crate::features::agents::AgentConnectionState;

#[derive(Clone, Copy, Debug)]
pub enum AgentLinkMsg {
    Windows(AgentConnectionState),
    OfferNative,
}

#[derive(Clone, Debug, Deserialize, guinea::Remote)]
#[remote(action)]
pub struct OpenNativeTaskManager;
