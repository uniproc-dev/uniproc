use guinea::prelude::Event;
use serde::Deserialize;

#[derive(Debug, Clone, Event, Deserialize, guinea::Remote)]
#[remote(event)]
pub struct AgentStateRequest;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize)]
pub enum AgentConnectionState {
    Disconnected,
    Connecting,
    Connected,
    WaitingRetry,
    GaveUp,
}
