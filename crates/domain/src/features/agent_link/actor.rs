use std::sync::Arc;
use std::time::Duration;

use app_contracts::features::agent_link::{AgentLinkMsg, AgentLinkState, InProcess, StartInProcess};
use app_contracts::features::agents::{
    AgentConnectionState, AgentStateRequest, WindowsActionRequest, WindowsActionResponse,
    WindowsAgentInProcess, WindowsAgentRuntimeEvent, WindowsMachineSample, WindowsReport, WindowsReportMessage,
};
use guinea::prelude::*;

use super::in_process::{InProcessAgent, InProcessStart, InProcessStartError};

pub struct Offer;

#[expect(non_upper_case_globals)]
impl Offer {
    pub const InProcessAfter: Duration = Duration::from_secs(5);
}

#[derive(Clone, Debug)]
pub struct OfferInProcessLater;

#[derive(Clone, Debug)]
pub struct InProcessOfferDue;

struct Reports;

#[expect(non_upper_case_globals)]
impl Reports {
    const RetryAfter: Duration = Duration::from_secs(1);
}

struct InProcessStarted(Result<Arc<dyn InProcessAgent>, InProcessStartError>);

struct InProcessReport(Option<WindowsReport>);

struct InProcessMachine(Option<WindowsMachineSample>);

pub struct AgentLinkActor {
    ui_port: Push<AgentLinkState>,
    start_in_process: InProcessStart,
    in_process: Option<Arc<dyn InProcessAgent>>,
    starting: bool,
    last: Option<AgentConnectionState>,
}

impl std::fmt::Debug for AgentLinkActor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AgentLinkActor")
            .field("in_process", &self.in_process.is_some())
            .field("starting", &self.starting)
            .field("last", &self.last)
            .finish()
    }
}

impl AgentLinkActor {
    pub fn new(ui_port: Push<AgentLinkState>, start_in_process: InProcessStart) -> Self {
        Self {
            ui_port,
            start_in_process,
            in_process: None,
            starting: false,
            last: None,
        }
    }

    fn announce_in_process(&self) {
        GlobalEventBus::publish(WindowsAgentRuntimeEvent {
            state: AgentConnectionState::Connected,
            latency_ms: None,
        });
    }
}

actor! {
    AgentLinkActor {
        handlers {
            WindowsAgentRuntimeEvent,
            StartInProcess,
            InProcessStarted,
            OfferInProcessLater,
            InProcessOfferDue,
            InProcessReport,
            InProcessMachine,
            WindowsActionRequest,
            AgentStateRequest,
        }
    }
}

#[handler]
fn on_windows_agent(this: &mut AgentLinkActor, WindowsAgentRuntimeEvent { state, .. }: WindowsAgentRuntimeEvent) {
    if this.last == Some(state) {
        return;
    }
    this.last = Some(state);
    this.ui_port.send(AgentLinkMsg::Windows(state));
}

#[handler]
fn start_in_process(this: &mut AgentLinkActor, _msg: StartInProcess, cx: Cx) {
    if this.starting || this.in_process.is_some() {
        return;
    }
    this.starting = true;
    this.ui_port.send(AgentLinkMsg::InProcess(InProcess::Starting));
    let start = this.start_in_process;
    cx.spawn_bg(async move { InProcessStarted(start().await) });
}

#[handler]
fn on_in_process_started(this: &mut AgentLinkActor, InProcessStarted(started): InProcessStarted, cx: Cx) {
    this.starting = false;
    match started {
        Ok(agent) => {
            tracing::info!("in-process agent started");
            this.in_process = Some(agent.clone());
            this.ui_port.send(AgentLinkMsg::InProcess(InProcess::Running));
            GlobalEventBus::publish(WindowsAgentInProcess);
            this.announce_in_process();
            cx.spawn_source(machine_samples(agent.clone()), InProcessMachine);
            cx.spawn_source(reports(agent), InProcessReport);
        }
        Err(InProcessStartError::NotElevated) => {
            tracing::warn!("in-process agent needs an elevated process");
            this.ui_port.send(AgentLinkMsg::InProcess(InProcess::NotElevated));
        }
        Err(InProcessStartError::Failed(error)) => {
            tracing::warn!(%error, "in-process agent did not start");
            this.ui_port.send(AgentLinkMsg::InProcess(InProcess::Failed));
        }
    }
}

fn reports(agent: Arc<dyn InProcessAgent>) -> impl futures::Stream<Item = Option<WindowsReport>> + Send + 'static {
    futures::stream::unfold(agent, |agent| async move {
        let report = match agent.clone().report().await {
            Ok(report) => report,
            Err(error) => {
                tracing::warn!(%error, "in-process agent did not report");
                tokio::time::sleep(Reports::RetryAfter).await;
                None
            }
        };
        Some((report, agent))
    })
}

fn machine_samples(
    agent: Arc<dyn InProcessAgent>,
) -> impl futures::Stream<Item = Option<WindowsMachineSample>> + Send + 'static {
    futures::stream::unfold(agent, |agent| async move {
        let sample = match agent.clone().machine().await {
            Ok(sample) => Some(sample),
            Err(error) => {
                tracing::warn!(%error, "in-process agent did not sample the machine");
                tokio::time::sleep(Reports::RetryAfter).await;
                None
            }
        };
        Some((sample, agent))
    })
}

#[handler]
fn on_machine(_this: &AgentLinkActor, InProcessMachine(sample): InProcessMachine) {
    if let Some(sample) = sample {
        GlobalEventBus::publish(sample);
    }
}

#[handler]
fn on_report(_this: &AgentLinkActor, InProcessReport(report): InProcessReport) {
    match report {
        Some(report) => GlobalEventBus::publish(WindowsReportMessage::Report(Arc::new(report))),
        None => tracing::debug!("no report from the in-process agent this time"),
    }
}

#[handler]
fn on_action(this: &AgentLinkActor, request: WindowsActionRequest, cx: Cx) {
    let Some(agent) = this.in_process.clone() else {
        return;
    };
    cx.spawn_bg_detached(async move {
        let code = agent.act(request.action).await;
        GlobalEventBus::publish(WindowsActionResponse::new(request.correlation_id, code));
    });
}

#[handler]
fn on_state_request(this: &AgentLinkActor, _msg: AgentStateRequest) {
    if this.in_process.is_some() {
        this.announce_in_process();
    }
}

#[handler]
async fn offer_in_process_later(ctx: AsyncContext<AgentLinkActor>, _msg: OfferInProcessLater) {
    let waited = ctx.until_gone(tokio::time::sleep(Offer::InProcessAfter)).await;
    if waited.is_some() {
        ctx.send(InProcessOfferDue);
    }
}

#[handler]
fn in_process_offer_due(this: &AgentLinkActor, _msg: InProcessOfferDue) {
    this.ui_port.send(AgentLinkMsg::OfferInProcess);
}
