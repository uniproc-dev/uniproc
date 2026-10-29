use crate::features::agents::actions::{ActsOnWindows, WindowsTransport};
use crate::features::agents::actor::{GenericAgentActor, Init, Ping};
use crate::features::agents::backend::{AgentBackend, InProcessError, Outdated};
use crate::features::agents::settings::AgentSettings;
use crate::features::agents::windows_feed::WindowsFeed;
use crate::features::settings::settings::GeneralSettings;
use amethystate::Field;
use app_contracts::features::agents::{
    AgentConnectionState, WindowsAction, WindowsAgentInProcess,
    WindowsAgentRuntimeEvent, WindowsMachineSample, WindowsProcessEvents, WindowsReport, WindowsReportMessage,
};
use futures::future::BoxFuture;
use futures::StreamExt;
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
        use uniproc_windows_agent::api::ServiceState;
        let restarting = matches!(&action, WindowsAction::ServiceRestart { .. });
        let (name, desired) = match &action {
            WindowsAction::ServiceStart { name } | WindowsAction::ServiceResume { name } | WindowsAction::ServiceRestart { name } => (name.clone(), ServiceState::Running),
            WindowsAction::ServiceStop { name } => (name.clone(), ServiceState::Stopped),
            WindowsAction::ServicePause { name } => (name.clone(), ServiceState::Paused),
            _ => return self.feed.act(action).await,
        };
        tokio::time::timeout(Duration::from_secs(55), async {
            let mut watch = match self.feed.agent().watch_service(&name).await {
                Ok(watch) => watch,
                Err(error) => {
                    tracing::warn!(%name, %error, "could not follow service operation");
                    return u32::MAX;
                }
            };
            if watch.next().await.is_none() {
                return u32::MAX;
            }
            let code = self.feed.act(action).await;
            if code != 0 { return code; }
            service_result(watch, desired, restarting).await
        }).await.unwrap_or(1460)
    }

    pub async fn process_events(&self) -> anyhow::Result<WindowsProcessEvents> {
        self.feed.process_events().await
    }
}

async fn service_result(
    mut watch: futures::stream::BoxStream<'static, uniproc_windows_agent::api::ServiceStatus>,
    desired: uniproc_windows_agent::api::ServiceState,
    restarting: bool,
) -> u32 {
    use uniproc_windows_agent::api::ServiceState;
    let mut saw_start = false;
    while let Some(status) = watch.next().await {
        saw_start |= status.state == ServiceState::StartPending;
        if status.state == desired { return 0; }
        if status.state == ServiceState::Stopped && desired != ServiceState::Stopped && (!restarting || saw_start) {
            return if status.exit_code == 0 { 1067 } else { status.exit_code };
        }
    }
    u32::MAX
}

#[cfg(test)]
mod service_operation_tests {
    use super::*;
    use uniproc_windows_agent::api::{ServiceState, ServiceStatus};

    fn statuses(states: &[ServiceState]) -> futures::stream::BoxStream<'static, ServiceStatus> {
        futures::stream::iter(states.iter().copied().map(|state| ServiceStatus { state, ..Default::default() }).collect::<Vec<_>>()).boxed()
    }

    #[tokio::test]
    async fn restart_waits_through_stopped_and_start_pending() {
        assert_eq!(service_result(statuses(&[ServiceState::StopPending, ServiceState::Stopped, ServiceState::StartPending, ServiceState::Running]), ServiceState::Running, true).await, 0);
        assert_eq!(service_result(statuses(&[ServiceState::StopPending, ServiceState::Stopped]), ServiceState::Running, true).await, u32::MAX);
    }

    #[tokio::test]
    async fn a_service_that_stops_during_start_is_a_failure() {
        assert_eq!(service_result(statuses(&[ServiceState::StartPending, ServiceState::Stopped]), ServiceState::Running, false).await, 1067);
        assert_eq!(service_result(statuses(&[ServiceState::StartPending, ServiceState::Stopped]), ServiceState::Running, true).await, 1067);
        let failed = futures::stream::iter([ServiceStatus { state: ServiceState::Stopped, exit_code: 5, ..Default::default() }]).boxed();
        assert_eq!(service_result(failed, ServiceState::Running, false).await, 5);
    }

    #[tokio::test]
    async fn stop_completes_only_after_the_stopped_status() {
        assert_eq!(service_result(statuses(&[ServiceState::StopPending]), ServiceState::Stopped, false).await, u32::MAX);
        assert_eq!(service_result(statuses(&[ServiceState::StopPending, ServiceState::Stopped]), ServiceState::Stopped, false).await, 0);
    }
}

impl ActsOnWindows for WindowsClient {
    fn act(&self, action: WindowsAction) -> BoxFuture<'static, u32> {
        let client = self.clone();
        Box::pin(async move { client.act(action).await })
    }
}

#[derive(Debug)]
pub struct WindowsBackend;

impl AgentBackend for WindowsBackend {
    type Client = WindowsClient;
    type RuntimeEvent = WindowsAgentRuntimeEvent;
    type ScanMessage = WindowsReportMessage;
    const NAME: &'static str = "Windows";
    const SUPPORTS_IN_PROCESS: bool = true;
    const STREAMS_MACHINE: bool = true;
    const STREAMS_PROCESS_EVENTS: bool = true;

    async fn connect(timeout: Duration, update_interval_ms: Field<u64>) -> anyhow::Result<Self::Client> {
        WindowsClient::connect(timeout, move || Duration::from_millis(update_interval_ms.get())).await
    }

    async fn connect_in_process(update_interval_ms: Field<u64>) -> Result<Self::Client, InProcessError> {
        WindowsClient::start_in_process(move || Duration::from_millis(update_interval_ms.get())).await
    }

    fn in_process_started(started: Result<(), &InProcessError>, cx: &AsyncContext<GenericAgentActor<Self>>) {
        cx.publish(match started {
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

    #[instrument(skip(client, cx), level = "debug", err)]
    async fn perform_scan(client: &Self::Client, cx: &AsyncContext<GenericAgentActor<Self>>) -> anyhow::Result<()> {
        match client.report().await? {
            Some(report) => cx.publish(WindowsReportMessage::Report(Arc::new(report))),
            None => tracing::debug!("process list kept moving under the metrics, skipping this update"),
        }
        Ok(())
    }

    async fn perform_machine_scan(client: &Self::Client, cx: &AsyncContext<GenericAgentActor<Self>>) -> anyhow::Result<()> {
        cx.publish(client.machine().await?);
        Ok(())
    }

    async fn perform_process_events(client: &Self::Client, cx: &AsyncContext<GenericAgentActor<Self>>) -> anyhow::Result<()> {
        cx.publish(client.process_events().await?);
        Ok(())
    }

    fn announce(client: Option<&Self::Client>, cx: &AsyncContext<GenericAgentActor<Self>>) {
        cx.publish(match client {
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

pub fn windows_agent_feature(app: &mut FeatureBuilder, store: &amethystate::Store) -> anyhow::Result<()> {
    let settings = store.try_settings::<AgentSettings>()?;
    let general = store.try_settings::<GeneralSettings>()?;
    let addr = app.spawn(GenericAgentActor::<WindowsBackend>::new(
        settings.connect_attempt_secs(),
        general.update_interval_ms(),
    ));

    app.every(settings.ping_period(), || Ping, addr.clone()).named("windows-agent-ping");

    addr.send(Init);

    Ok(())
}
