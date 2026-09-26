use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::Duration;

use app_contracts::features::agents::{
    AgentConnectionState, AgentStateRequest, ScanTick, WindowsAgentRuntimeEvent, WindowsReport,
    WindowsReportMessage,
};
use domain::features::agents::actor::{GenericAgentActor, Init, Ping};
use domain::features::agents::backend::AgentBackend;
use domain::features::agents::settings::AgentSettings;
use guinea::prelude::*;

static UP: AtomicBool = AtomicBool::new(false);
static CONNECTS: AtomicU32 = AtomicU32::new(0);
static REPORT: std::sync::Mutex<Option<WindowsReport>> = std::sync::Mutex::new(None);

pub fn reset(up: bool) {
    UP.store(up, Ordering::SeqCst);
    CONNECTS.store(0, Ordering::SeqCst);
    *REPORT.lock().unwrap() = None;
}

pub fn set_up(up: bool) {
    UP.store(up, Ordering::SeqCst);
}

pub fn connects() -> u32 {
    CONNECTS.load(Ordering::SeqCst)
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

    async fn connect(_timeout_secs: u64) -> anyhow::Result<()> {
        CONNECTS.fetch_add(1, Ordering::SeqCst);
        up()
    }

    async fn ping(_client: &()) -> anyhow::Result<i32> {
        up().map(|()| 1)
    }

    async fn perform_scan(_client: &()) -> anyhow::Result<()> {
        up()?;
        let report = REPORT.lock().unwrap().clone();
        if let Some(report) = report {
            GlobalEventBus::publish(WindowsReportMessage::Report(std::sync::Arc::new(report)));
        }
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
        let settings = AgentSettings::new()?;
        let addr = app.spawn(GenericAgentActor::<FakeAgent>::new(settings.connect_attempt_secs()));

        app.every(Duration::from_secs(1), &addr, || Ping);
        app.repeat(Duration::from_millis(500), || GlobalEventBus::publish(ScanTick));
        addr.subscribe_on::<ScanTick>(Bus::Global);
        addr.subscribe_on::<AgentStateRequest>(Bus::Global);
        addr.send(Init);

        Ok(())
    }
}
