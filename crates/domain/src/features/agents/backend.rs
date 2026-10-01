use app_contracts::features::agents::AgentConnectionState;
use guinea::prelude::*;

#[derive(Debug)]
pub struct Outdated(pub String);

impl std::fmt::Display for Outdated {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "the agent is too old for this client: {}", self.0)
    }
}

impl std::error::Error for Outdated {}

pub trait AgentBackend: std::fmt::Debug + Send + Sync + 'static {
    type Client: Clone + std::fmt::Debug + Send + Sync + 'static;
    type RuntimeEvent: Event;
    type ScanMessage: Event;

    const NAME: &'static str;
    const STREAMS_MACHINE: bool = false;

    fn connect(timeout_secs: u64) -> impl Future<Output=anyhow::Result<Self::Client>> + Send;
    fn ping(client: &Self::Client) -> impl Future<Output=anyhow::Result<i32>> + Send;
    fn perform_scan(client: &Self::Client) -> impl Future<Output=anyhow::Result<()>> + Send;

    fn perform_machine_scan(_client: &Self::Client) -> impl Future<Output=anyhow::Result<()>> + Send {
        std::future::pending()
    }

    fn create_runtime_event(state: AgentConnectionState, latency_ms: Option<i32>) -> Self::RuntimeEvent;

    fn scan_unavailable(state: AgentConnectionState) -> Self::ScanMessage;
}
