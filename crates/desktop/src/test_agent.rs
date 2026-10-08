use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::Duration;

use amethystate::Field;
use app_contracts::features::agents::{
    AgentConnectionState, AgentStateRequest, RunWindowsAgentInProcess, WindowsAction, WindowsAgentInProcess,
    WindowsAgentRuntimeEvent, WindowsProcessEvents, WindowsReport, WindowsReportMessage,
};
use domain::features::agent_link::{AgentLinkDeps, Elevation, RelaunchError};
use domain::features::agents::actions::{self, ActsOnWindows, WindowsTransport};
use domain::features::agents::actor::{GenericAgentActor, Init, Ping};
use domain::features::agents::backend::{AgentBackend, InProcessError, Outdated};
use domain::features::agents::settings::AgentSettings;
use domain::features::settings::settings::GeneralSettings;
use futures::future::BoxFuture;
use guinea::prelude::*;
use guinea_plugin_store::StoreAccess;

struct Pace;

#[expect(non_upper_case_globals)]
impl Pace {
    const Report: Duration = Duration::from_millis(500);
}

static UP: AtomicBool = AtomicBool::new(false);
static OUTDATED: AtomicBool = AtomicBool::new(false);
static DROPS: AtomicBool = AtomicBool::new(false);
static CONNECTS: AtomicU32 = AtomicU32::new(0);
static PINGS: AtomicU32 = AtomicU32::new(0);
static REPORT: std::sync::Mutex<Option<WindowsReport>> = std::sync::Mutex::new(None);
static ELEVATED: AtomicBool = AtomicBool::new(false);
static ASKED_AT_START: AtomicBool = AtomicBool::new(false);
static RELAUNCH_REFUSED: AtomicBool = AtomicBool::new(false);
static IN_PROCESS_FAILS: AtomicBool = AtomicBool::new(false);
static RELAUNCHES: AtomicU32 = AtomicU32::new(0);
static CLOSES: AtomicU32 = AtomicU32::new(0);
static IN_PROCESS_STARTS: AtomicU32 = AtomicU32::new(0);
static IN_PROCESS_REPORTS: AtomicU32 = AtomicU32::new(0);
static IN_PROCESS_ACTIONS: std::sync::Mutex<Vec<WindowsAction>> = std::sync::Mutex::new(Vec::new());
static EVENTS: std::sync::Mutex<Option<WindowsProcessEvents>> = std::sync::Mutex::new(None);

pub fn reset(up: bool) {
    UP.store(up, Ordering::SeqCst);
    OUTDATED.store(false, Ordering::SeqCst);
    DROPS.store(false, Ordering::SeqCst);
    CONNECTS.store(0, Ordering::SeqCst);
    PINGS.store(0, Ordering::SeqCst);
    *REPORT.lock().unwrap() = None;
    ELEVATED.store(false, Ordering::SeqCst);
    ASKED_AT_START.store(false, Ordering::SeqCst);
    RELAUNCH_REFUSED.store(false, Ordering::SeqCst);
    IN_PROCESS_FAILS.store(false, Ordering::SeqCst);
    RELAUNCHES.store(0, Ordering::SeqCst);
    CLOSES.store(0, Ordering::SeqCst);
    IN_PROCESS_STARTS.store(0, Ordering::SeqCst);
    IN_PROCESS_REPORTS.store(0, Ordering::SeqCst);
    IN_PROCESS_ACTIONS.lock().unwrap().clear();
    *EVENTS.lock().unwrap() = None;
}

pub fn tell(events: WindowsProcessEvents) {
    *EVENTS.lock().unwrap() = Some(events);
}

async fn told() -> Option<WindowsProcessEvents> {
    let events = EVENTS.lock().unwrap().take();
    if events.is_none() {
        tokio::time::sleep(Pace::Report).await;
    }
    events
}

pub fn set_elevated(elevated: bool) {
    ELEVATED.store(elevated, Ordering::SeqCst);
}

pub fn set_asked_at_start(asked: bool) {
    ASKED_AT_START.store(asked, Ordering::SeqCst);
}

pub fn set_in_process_fails(fails: bool) {
    IN_PROCESS_FAILS.store(fails, Ordering::SeqCst);
}

pub fn set_relaunch_refused(refused: bool) {
    RELAUNCH_REFUSED.store(refused, Ordering::SeqCst);
}

pub fn relaunches() -> u32 {
    RELAUNCHES.load(Ordering::SeqCst)
}

pub fn closes() -> u32 {
    CLOSES.load(Ordering::SeqCst)
}

pub fn elevation() -> Elevation {
    Elevation {
        elevated: || ELEVATED.load(Ordering::SeqCst),
        asked_at_start: || ASKED_AT_START.load(Ordering::SeqCst),
        relaunch: || {
            RELAUNCHES.fetch_add(1, Ordering::SeqCst);
            let refused = RELAUNCH_REFUSED.load(Ordering::SeqCst);
            Box::pin(async move {
                guinea::core::executor::random_delay().await;
                if refused { Err(RelaunchError::Refused) } else { Ok(()) }
            })
        },
        close: || {
            CLOSES.fetch_add(1, Ordering::SeqCst);
        },
    }
}

pub fn agent_link() -> AgentLinkDeps {
    AgentLinkDeps {
        elevation: elevation(),
    }
}

pub fn in_process_starts() -> u32 {
    IN_PROCESS_STARTS.load(Ordering::SeqCst)
}

pub fn in_process_reports() -> u32 {
    IN_PROCESS_REPORTS.load(Ordering::SeqCst)
}

pub fn in_process_actions() -> Vec<WindowsAction> {
    IN_PROCESS_ACTIONS.lock().unwrap().clone()
}

#[derive(Clone, Copy, Debug)]
pub struct FakeClient {
    in_process: bool,
}

impl FakeClient {
    fn up(self) -> anyhow::Result<()> {
        match self.in_process {
            true => Ok(()),
            false => up(),
        }
    }
}

impl ActsOnWindows for FakeClient {
    fn act(&self, action: WindowsAction) -> BoxFuture<'static, u32> {
        if self.in_process {
            IN_PROCESS_ACTIONS.lock().unwrap().push(action);
        }
        Box::pin(async { 0 })
    }
}

pub fn set_up(up: bool) {
    UP.store(up, Ordering::SeqCst);
}

pub fn set_outdated(outdated: bool) {
    OUTDATED.store(outdated, Ordering::SeqCst);
}

pub fn set_drops(drops: bool) {
    DROPS.store(drops, Ordering::SeqCst);
}

pub fn connects() -> u32 {
    CONNECTS.load(Ordering::SeqCst)
}

pub fn pings() -> u32 {
    PINGS.load(Ordering::SeqCst)
}

pub fn serve(report: WindowsReport) {
    *REPORT.lock().unwrap() = Some(report);
}

fn up() -> anyhow::Result<()> {
    if UP.load(Ordering::SeqCst) {
        Ok(())
    } else {
        anyhow::bail!("the agent is down")
    }
}

#[derive(Debug)]
pub struct FakeAgent;

impl AgentBackend for FakeAgent {
    type Client = FakeClient;
    type RuntimeEvent = WindowsAgentRuntimeEvent;
    type ScanMessage = WindowsReportMessage;

    const NAME: &'static str = "Fake";
    const STREAMS_PROCESS_EVENTS: bool = true;

    async fn connect(_timeout: Duration, _update_interval_ms: Field<u64>) -> anyhow::Result<FakeClient> {
        CONNECTS.fetch_add(1, Ordering::SeqCst);
        if OUTDATED.load(Ordering::SeqCst) {
            return Err(Outdated("windows 2.1.0".into()).into());
        }
        up().map(|()| FakeClient { in_process: false })
    }

    async fn connect_in_process(_update_interval_ms: Field<u64>) -> Result<FakeClient, InProcessError> {
        IN_PROCESS_STARTS.fetch_add(1, Ordering::SeqCst);
        if IN_PROCESS_FAILS.load(Ordering::SeqCst) {
            return Err(InProcessError::Failed("the ETW session is taken".into()));
        }
        match ELEVATED.load(Ordering::SeqCst) {
            true => Ok(FakeClient { in_process: true }),
            false => Err(InProcessError::NotElevated),
        }
    }

    fn in_process_started(started: Result<(), &InProcessError>) {
        GlobalEventBus::publish(match started {
            Ok(()) => WindowsAgentInProcess::Running,
            Err(InProcessError::NotElevated) => WindowsAgentInProcess::NotElevated,
            Err(_) => WindowsAgentInProcess::Failed,
        });
    }

    async fn ping(client: &FakeClient) -> anyhow::Result<i32> {
        PINGS.fetch_add(1, Ordering::SeqCst);
        client.up().map(|()| 1)
    }

    async fn perform_scan(client: &FakeClient) -> anyhow::Result<()> {
        client.up()?;
        let report = match client.in_process {
            true => {
                IN_PROCESS_REPORTS.fetch_add(1, Ordering::SeqCst);
                Some(REPORT.lock().unwrap().clone().unwrap_or_default())
            }
            false => {
                if DROPS.load(Ordering::SeqCst) {
                    anyhow::bail!("the agent dropped the watch");
                }
                REPORT.lock().unwrap().clone()
            }
        };
        if let Some(report) = report {
            GlobalEventBus::publish(WindowsReportMessage::Report(Arc::new(report)));
        }
        tokio::time::sleep(Pace::Report).await;
        Ok(())
    }

    async fn perform_process_events(client: &FakeClient) -> anyhow::Result<()> {
        client.up()?;
        if let Some(events) = told().await {
            GlobalEventBus::publish(events);
        }
        Ok(())
    }

    fn announce(client: Option<&FakeClient>) {
        GlobalEventBus::publish(match client {
            Some(client) => WindowsTransport::Connected(Arc::new(*client)),
            None => WindowsTransport::Lost,
        });
    }

    fn create_runtime_event(state: AgentConnectionState, latency_ms: Option<i32>) -> WindowsAgentRuntimeEvent {
        WindowsAgentRuntimeEvent { state, latency_ms }
    }

    fn scan_unavailable(state: AgentConnectionState) -> WindowsReportMessage {
        WindowsReportMessage::Unavailable(state)
    }
}

pub struct FakeAgentFeature;

impl AppFeature for FakeAgentFeature {
    type Exports = ();

    fn install(self, app: &mut FeatureBuilder) -> anyhow::Result<()> {
        let settings = app.settings::<AgentSettings>();
        let general = app.settings::<GeneralSettings>();
        actions::install(app);
        let addr = app.spawn(GenericAgentActor::<FakeAgent>::new(
            settings.connect_attempt_secs(),
            general.update_interval_ms(),
        ));

        app.every(settings.ping_period(), &addr, || Ping);
        addr.subscribe_on::<AgentStateRequest>(Bus::Global);
        addr.subscribe_on::<RunWindowsAgentInProcess>(Bus::Global);
        addr.send(Init);

        Ok(())
    }
}
