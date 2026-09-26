use app_contracts::features::agents::AgentConnectionState;
use guinea::prelude::*;

pub trait AgentBackend: std::fmt::Debug + Send + Sync + 'static {
    type Client: Clone + std::fmt::Debug + Send + Sync + 'static;
    type RuntimeEvent: Event;
    type ScanMessage: Event;

    const NAME: &'static str;

    fn connect(timeout_secs: u64) -> impl Future<Output=anyhow::Result<Self::Client>> + Send;
    fn ping(client: &Self::Client) -> impl Future<Output=anyhow::Result<i32>> + Send;
    fn perform_scan(client: &Self::Client) -> impl Future<Output=anyhow::Result<()>> + Send;

    fn create_runtime_event(state: AgentConnectionState, latency_ms: Option<i32>) -> Self::RuntimeEvent;

    fn scan_unavailable(state: AgentConnectionState) -> Self::ScanMessage;
}
