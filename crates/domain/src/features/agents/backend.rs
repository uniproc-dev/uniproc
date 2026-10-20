use amethystate::Field;
use app_contracts::features::agents::{AgentConnectionState, RemoteScanResult, WindowsAgentRuntimeEvent, WindowsReportMessage, WslAgentRuntimeEvent};
use guinea::prelude::*;
use std::time::Duration;

#[derive(Debug)]
pub struct Outdated(pub String);

impl std::fmt::Display for Outdated {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "the agent is too old for this client: {}", self.0)
    }
}

impl std::error::Error for Outdated {}

#[derive(Debug)]
pub enum InProcessError {
    Unsupported,
    NotElevated,
    Failed(String),
}

impl std::fmt::Display for InProcessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unsupported => f.write_str("this agent does not run in process"),
            Self::NotElevated => f.write_str("monitoring in process needs an elevated process"),
            Self::Failed(error) => write!(f, "monitoring in process did not start: {error}"),
        }
    }
}

pub enum AgentEvent {
    WindowsState(WindowsAgentRuntimeEvent),
    WslState(WslAgentRuntimeEvent),
    WindowsReport(WindowsReportMessage),
    WslReport(RemoteScanResult),
}

impl From<WindowsAgentRuntimeEvent> for AgentEvent {
    fn from(event: WindowsAgentRuntimeEvent) -> Self { Self::WindowsState(event) }
}

impl From<WslAgentRuntimeEvent> for AgentEvent {
    fn from(event: WslAgentRuntimeEvent) -> Self { Self::WslState(event) }
}

impl From<WindowsReportMessage> for AgentEvent {
    fn from(event: WindowsReportMessage) -> Self { Self::WindowsReport(event) }
}

impl From<RemoteScanResult> for AgentEvent {
    fn from(event: RemoteScanResult) -> Self { Self::WslReport(event) }
}

pub trait AgentBackend: std::fmt::Debug + Send + Sync + Sized + 'static {
    type Client: Clone + std::fmt::Debug + Send + Sync + 'static;
    type RuntimeEvent: Event + Into<AgentEvent>;
    type ScanMessage: Event + Into<AgentEvent>;

    const NAME: &'static str;
    const SUPPORTS_IN_PROCESS: bool = false;
    const STREAMS_MACHINE: bool = false;
    const STREAMS_PROCESS_EVENTS: bool = false;

    fn connect(timeout: Duration, update_interval_ms: Field<u64>) -> impl Future<Output=anyhow::Result<Self::Client>> + Send;

    fn connect_in_process(
        _update_interval_ms: Field<u64>,
    ) -> impl Future<Output = Result<Self::Client, InProcessError>> + Send {
        std::future::ready(Err(InProcessError::Unsupported))
    }

    fn in_process_started(_started: Result<(), &InProcessError>, _cx: &AsyncContext<super::actor::GenericAgentActor<Self>>) {}
    fn ping(client: &Self::Client) -> impl Future<Output=anyhow::Result<i32>> + Send;
    fn perform_scan(client: &Self::Client, cx: &AsyncContext<super::actor::GenericAgentActor<Self>>) -> impl Future<Output=anyhow::Result<()>> + Send;

    fn perform_machine_scan(_client: &Self::Client, _cx: &AsyncContext<super::actor::GenericAgentActor<Self>>) -> impl Future<Output=anyhow::Result<()>> + Send {
        std::future::pending()
    }

    fn perform_process_events(_client: &Self::Client, _cx: &AsyncContext<super::actor::GenericAgentActor<Self>>) -> impl Future<Output=anyhow::Result<()>> + Send {
        std::future::pending()
    }

    fn announce(_client: Option<&Self::Client>, _cx: &AsyncContext<super::actor::GenericAgentActor<Self>>) {}

    fn create_runtime_event(state: AgentConnectionState, latency_ms: Option<i32>) -> Self::RuntimeEvent;

    fn scan_unavailable(state: AgentConnectionState) -> Self::ScanMessage;
}
