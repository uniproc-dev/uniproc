use crate::features::agents::actions::{ActsOnWindows, WindowsTransport};
use crate::features::agents::actor::{GenericAgentActor, Init, Ping};
use crate::features::agents::backend::{AgentBackend, InProcessError, Outdated};
use crate::features::agents::settings::AgentSettings;
use crate::features::agents::windows_feed::WindowsFeed;
use crate::features::settings::settings::GeneralSettings;
use amethystate::Field;
use app_contracts::features::agents::{
    AgentConnectionState, AgentStateRequest, RunWindowsAgentInProcess, WindowsAction, WindowsAgentInProcess,
    WindowsAgentRuntimeEvent, WindowsMachineSample, WindowsProcessEvents, WindowsReport, WindowsReportMessage,
};
use futures::future::BoxFuture;
use guinea::prelude::*;
use guinea_plugin_store::StoreAccess;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::instrument;
use uniproc_protocol::WINDOWS_AGENT_SERVICE;
use uniproc_windows_agent::agent::Agent;
pub use uniproc_windows_agent::api::{SERVICE_DISPLAY_NAME, SERVICE_NAME};
use uniproc_windows_agent::local::StartError;
use uniproc_windows_agent::remote::Remote;

fn agent_service() -> String {
    #[cfg(debug_assertions)]
    if let Some(service) = std::env::var_os("UNIPROC_AGENT_PIPE") {
        return service.to_string_lossy().into_owned();
    }
    WINDOWS_AGENT_SERVICE.to_string()
}

#[derive(Clone)]
pub struct WindowsClient {
    feed: Arc<WindowsFeed>,
}

impl std::fmt::Debug for WindowsClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WindowsClient").finish_non_exhaustive()
    }
}

impl WindowsClient {
    pub async fn connect(
        give_up_after: Duration,
        interval: impl Fn() -> Duration + Send + Sync + 'static,
    ) -> anyhow::Result<Self> {
        let remote = Remote::connect_to(&agent_service(), give_up_after).await?;
        if !remote.can_watch() {
            return Err(Outdated(remote.agent_version().to_string()).into());
        }
        Ok(Self::over(Agent::Remote(remote), interval))
    }

    pub async fn start_in_process(
        interval: impl Fn() -> Duration + Send + Sync + 'static,
    ) -> Result<Self, InProcessError> {
        match tokio::task::spawn_blocking(Agent::local).await {
            Ok(Ok(agent)) => Ok(Self::over(agent, interval)),
            Ok(Err(StartError::NotElevated)) => Err(InProcessError::NotElevated),
            Ok(Err(error)) => Err(InProcessError::Failed(error.to_string())),
            Err(error) => Err(InProcessError::Failed(error.to_string())),
        }
    }

    fn over(agent: Agent, interval: impl Fn() -> Duration + Send + Sync + 'static) -> Self {
        Self {
            feed: Arc::new(WindowsFeed::new(agent, interval)),
        }
    }

    pub async fn ping(&self) -> anyhow::Result<()> {
        self.feed.agent().ping().await
    }

    pub async fn report(&self) -> anyhow::Result<Option<WindowsReport>> {
        self.feed.report().await
    }

    pub async fn machine(&self) -> anyhow::Result<WindowsMachineSample> {
        self.feed.machine().await
    }

    pub async fn act(&self, action: WindowsAction) -> u32 {
        self.feed.act(action).await
    }

    pub async fn process_events(&self) -> anyhow::Result<WindowsProcessEvents> {
        self.feed.process_events().await
    }
}

impl ActsOnWindows for WindowsClient {
    fn act(&self, action: WindowsAction) -> BoxFuture<'static, u32> {
        let client = self.clone();
        Box::pin(async move { client.feed.act(action).await })
    }
}

#[derive(Debug)]
pub struct WindowsBackend;

impl AgentBackend for WindowsBackend {
    type Client = WindowsClient;
    type RuntimeEvent = WindowsAgentRuntimeEvent;
    type ScanMessage = WindowsReportMessage;
    const NAME: &'static str = "Windows";
    const STREAMS_MACHINE: bool = true;
    const STREAMS_PROCESS_EVENTS: bool = true;

    async fn connect(timeout: Duration, update_interval_ms: Field<u64>) -> anyhow::Result<Self::Client> {
        WindowsClient::connect(timeout, move || Duration::from_millis(update_interval_ms.get())).await
    }

    async fn connect_in_process(update_interval_ms: Field<u64>) -> Result<Self::Client, InProcessError> {
        WindowsClient::start_in_process(move || Duration::from_millis(update_interval_ms.get())).await
    }

    fn in_process_started(started: Result<(), &InProcessError>) {
        GlobalEventBus::publish(match started {
            Ok(()) => WindowsAgentInProcess::Running,
            Err(InProcessError::NotElevated) => WindowsAgentInProcess::NotElevated,
            Err(_) => WindowsAgentInProcess::Failed,
        });
    }

    async fn ping(client: &Self::Client) -> anyhow::Result<i32> {
        let start = Instant::now();
        client.ping().await?;
        Ok(start.elapsed().as_millis() as i32)
    }

    #[instrument(skip(client), level = "debug", err)]
    async fn perform_scan(client: &Self::Client) -> anyhow::Result<()> {
        match client.report().await? {
            Some(report) => GlobalEventBus::publish(WindowsReportMessage::Report(Arc::new(report))),
            None => tracing::debug!("process list kept moving under the metrics, skipping this update"),
        }
        Ok(())
    }

    async fn perform_machine_scan(client: &Self::Client) -> anyhow::Result<()> {
        GlobalEventBus::publish(client.machine().await?);
        Ok(())
    }

    async fn perform_process_events(client: &Self::Client) -> anyhow::Result<()> {
        GlobalEventBus::publish(client.process_events().await?);
        Ok(())
    }

    fn announce(client: Option<&Self::Client>) {
        GlobalEventBus::publish(match client {
            Some(client) => WindowsTransport::Connected(Arc::new(client.clone())),
            None => WindowsTransport::Lost,
        });
    }

    fn create_runtime_event(state: AgentConnectionState, latency: Option<i32>) -> Self::RuntimeEvent {
        WindowsAgentRuntimeEvent { state, latency_ms: latency }
    }

    fn scan_unavailable(state: AgentConnectionState) -> Self::ScanMessage {
        WindowsReportMessage::Unavailable(state)
    }
}

pub fn windows_agent_feature(app: &mut FeatureBuilder) -> anyhow::Result<()> {
    let settings = app.settings::<AgentSettings>();
    let general = app.settings::<GeneralSettings>();
    let addr = app.spawn(GenericAgentActor::<WindowsBackend>::new(
        settings.connect_attempt_secs(),
        general.update_interval_ms(),
    ));

    app.every(settings.ping_period(), &addr, || Ping).named("windows-agent-ping");

    addr.subscribe_on::<AgentStateRequest>(Bus::Global);
    addr.subscribe_on::<RunWindowsAgentInProcess>(Bus::Global);
    addr.send(Init);

    Ok(())
}
