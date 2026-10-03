use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::Duration;

use amethystate::Field;
use app_contracts::features::agents::{
    AgentConnectionState, AgentStateRequest, WindowsAction, WindowsAgentInProcess,
    WindowsAgentRuntimeEvent, WindowsMachineSample, WindowsReport, WindowsReportMessage,
};
use domain::features::agent_link::{InProcessAgent, InProcessStartError};
use domain::features::agents::actions;
use domain::features::agents::actor::{GenericAgentActor, Init, Ping};
use domain::features::agents::backend::{AgentBackend, Outdated};
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
static IN_PROCESS_STARTS: AtomicU32 = AtomicU32::new(0);
static IN_PROCESS_REPORTS: AtomicU32 = AtomicU32::new(0);
static IN_PROCESS_ACTIONS: std::sync::Mutex<Vec<WindowsAction>> = std::sync::Mutex::new(Vec::new());

pub fn reset(up: bool) {
    UP.store(up, Ordering::SeqCst);
    OUTDATED.store(false, Ordering::SeqCst);
    DROPS.store(false, Ordering::SeqCst);
    CONNECTS.store(0, Ordering::SeqCst);
    PINGS.store(0, Ordering::SeqCst);
    *REPORT.lock().unwrap() = None;
    ELEVATED.store(false, Ordering::SeqCst);
    IN_PROCESS_STARTS.store(0, Ordering::SeqCst);
    IN_PROCESS_REPORTS.store(0, Ordering::SeqCst);
    IN_PROCESS_ACTIONS.lock().unwrap().clear();
}

pub fn set_elevated(elevated: bool) {
    ELEVATED.store(elevated, Ordering::SeqCst);
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

struct FakeInProcess;

impl InProcessAgent for FakeInProcess {
    fn report(self: Arc<Self>) -> BoxFuture<'static, anyhow::Result<Option<WindowsReport>>> {
        IN_PROCESS_REPORTS.fetch_add(1, Ordering::SeqCst);
        let report = REPORT.lock().unwrap().clone().unwrap_or_default();
        Box::pin(async {
            tokio::time::sleep(Pace::Report).await;
            Ok(Some(report))
        })
    }

    fn machine(self: Arc<Self>) -> BoxFuture<'static, anyhow::Result<WindowsMachineSample>> {
        Box::pin(std::future::pending())
    }

    fn act(self: Arc<Self>, action: WindowsAction) -> BoxFuture<'static, u32> {
        IN_PROCESS_ACTIONS.lock().unwrap().push(action);
        Box::pin(async { 0 })
    }
}

pub fn start_in_process(
    _update_interval_ms: Field<u64>,
) -> BoxFuture<'static, Result<Arc<dyn InProcessAgent>, InProcessStartError>> {
    IN_PROCESS_STARTS.fetch_add(1, Ordering::SeqCst);
    let elevated = ELEVATED.load(Ordering::SeqCst);
    Box::pin(async move {
        if elevated {
            Ok(Arc::new(FakeInProcess) as Arc<dyn InProcessAgent>)
        } else {
            Err(InProcessStartError::NotElevated)
        }
    })
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
    type Client = ();
    type RuntimeEvent = WindowsAgentRuntimeEvent;
    type ScanMessage = WindowsReportMessage;

    const NAME: &'static str = "Fake";

    async fn connect(_timeout: Duration, _update_interval_ms: Field<u64>) -> anyhow::Result<()> {
        CONNECTS.fetch_add(1, Ordering::SeqCst);
        if OUTDATED.load(Ordering::SeqCst) {
            return Err(Outdated("windows 2.1.0".into()).into());
        }
        up()
    }

    async fn ping(_client: &()) -> anyhow::Result<i32> {
        PINGS.fetch_add(1, Ordering::SeqCst);
        up().map(|()| 1)
    }

    async fn perform_scan(_client: &()) -> anyhow::Result<()> {
        up()?;
        if DROPS.load(Ordering::SeqCst) {
            anyhow::bail!("the agent dropped the watch");
        }
        let report = REPORT.lock().unwrap().clone();
        if let Some(report) = report {
            GlobalEventBus::publish(WindowsReportMessage::Report(std::sync::Arc::new(report)));
        }
        tokio::time::sleep(Pace::Report).await;
        Ok(())
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
    fn install(self, app: &mut FeatureBuilder) -> anyhow::Result<()> {
        let settings = app.settings::<AgentSettings>()?;
        let general = app.settings::<GeneralSettings>()?;
        actions::install(app);
        let addr = app.spawn(GenericAgentActor::<FakeAgent>::new(
            settings.connect_attempt_secs(),
            general.update_interval_ms(),
        ));

        app.every(settings.ping_period(), &addr, || Ping);
        addr.subscribe_on::<AgentStateRequest>(Bus::Global);
        addr.subscribe_on::<WindowsAgentInProcess>(Bus::Global);
        addr.send(Init);

        Ok(())
    }
}
