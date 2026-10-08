use serde::Deserialize;

use crate::features::agents::AgentConnectionState;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
pub enum InProcess {
    #[default]
    Off,
    Starting,
    Elevating,
    Running,
    NotElevated,
    Failed,
}

#[derive(Clone, Copy, Debug)]
pub enum AgentLinkMsg {
    Windows(AgentConnectionState),
    OfferInProcess,
    InProcess(InProcess),
    Elevated(bool),
}

#[derive(Clone, Debug, Deserialize, guinea::Remote)]
#[remote(action)]
pub struct StartInProcess;
