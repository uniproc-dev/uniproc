use crate::features::agents::actor::{GenericAgentActor, Init, Ping};
use crate::features::agents::backend::{AgentBackend, Outdated};
use crate::features::agents::settings::AgentSettings;
use crate::features::agents::windows_feed::WindowsFeed;
use crate::features::settings::settings::GeneralSettings;
use app_contracts::features::agents::{
    AgentConnectionState, AgentStateRequest, WindowsAction, WindowsActionRequest,
    WindowsAgentInProcess, WindowsAgentRuntimeEvent, WindowsMachineSample, WindowsReport, WindowsReportMessage,
};
use app_contracts::features::settings::UpdateInterval;
use guinea::prelude::*;
use guinea::ratelimit;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::instrument;
pub use uniproc_protocol::WINDOWS_AGENT_SERVICE;
use uniproc_windows_agent::agent::Agent;
use uniproc_windows_agent::remote::Remote;

pub const AGENT_SERVICE_DISPLAY_NAME: &str = "Uniproc Process Monitor";

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
        Ok(Self {
            feed: Arc::new(WindowsFeed::new(Agent::Remote(remote), interval)),
        })
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
}

#[derive(Debug)]
pub struct WindowsBackend;

impl AgentBackend for WindowsBackend {
    type Client = WindowsClient;
    type RuntimeEvent = WindowsAgentRuntimeEvent;
    type ScanMessage = WindowsReportMessage;
    const NAME: &'static str = "Windows";
    const STREAMS_MACHINE: bool = true;

    async fn connect(timeout: u64) -> anyhow::Result<Self::Client> {
        let interval = GeneralSettings::new()?.update_interval_ms();
        WindowsClient::connect(Duration::from_secs(timeout), move || UpdateInterval::clamp(interval.get())).await
    }

    async fn ping(client: &Self::Client) -> anyhow::Result<i32> {
        let start = Instant::now();
        client.ping().await?;
        Ok(start.elapsed().as_millis() as i32)
    }

    #[instrument(skip(client), level = "debug", err)]
    async fn perform_scan(client: &Self::Client) -> anyhow::Result<()> {
        match client.report().await? {
            Some(report) => {
                GlobalEventBus::publish(WindowsReportMessage::Report(Arc::new(report)));
                ratelimit!(3600, info!("Report published to event bus"));
            }
            None => tracing::debug!("process list kept moving under the metrics, skipping this update"),
        }
        Ok(())
    }

    async fn perform_machine_scan(client: &Self::Client) -> anyhow::Result<()> {
        GlobalEventBus::publish(client.machine().await?);
        Ok(())
    }

    fn create_runtime_event(state: AgentConnectionState, latency: Option<i32>) -> Self::RuntimeEvent {
        WindowsAgentRuntimeEvent { state, latency_ms: latency }
    }

    fn scan_unavailable(state: AgentConnectionState) -> Self::ScanMessage {
        WindowsReportMessage::Unavailable(state)
    }
}

pub fn windows_agent_feature(app: &mut FeatureBuilder) -> anyhow::Result<()> {
    let settings = AgentSettings::new()?;
    let ping_interval = settings.ping_interval_ms();

    let addr = app.spawn(GenericAgentActor::<WindowsBackend>::new(
        settings.connect_attempt_secs(),
    ));

    app.every(
        Period::varying(move || Duration::from_millis(ping_interval.get())),
        &addr,
        || Ping,
    )
    .named("windows-agent-ping");

    addr.subscribe_on::<WindowsActionRequest>(Bus::Global);
    addr.subscribe_on::<AgentStateRequest>(Bus::Global);
    addr.subscribe_on::<WindowsAgentInProcess>(Bus::Global);
    addr.send(Init);

    Ok(())
}
