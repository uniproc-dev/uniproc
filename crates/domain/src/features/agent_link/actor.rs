use std::sync::Arc;
use std::time::Duration;

use app_contracts::features::agent_link::{AgentLinkMsg, AgentLinkState, InProcess, StartInProcess};
use app_contracts::features::agents::{
    AgentConnectionState, AgentStateRequest, ScanTick, WindowsActionRequest, WindowsActionResponse,
    WindowsAgentInProcess, WindowsAgentRuntimeEvent, WindowsReportMessage,
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

struct InProcessStarted(Result<Arc<dyn InProcessAgent>, InProcessStartError>);

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
            ScanTick,
            WindowsActionRequest,
            AgentStateRequest,
        }
    }
}

#[handler]
fn on_windows_agent(this: &mut AgentLinkActor, msg: WindowsAgentRuntimeEvent) {
    let state = msg.state;
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
fn on_in_process_started(this: &mut AgentLinkActor, InProcessStarted(started): InProcessStarted) {
    this.starting = false;
    match started {
        Ok(agent) => {
            tracing::info!("in-process agent started");
            this.in_process = Some(agent);
            this.ui_port.send(AgentLinkMsg::InProcess(InProcess::Running));
            GlobalEventBus::publish(WindowsAgentInProcess);
            this.announce_in_process();
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

#[handler]
fn on_scan_tick(this: &AgentLinkActor, _msg: ScanTick) {
    if let Some(agent) = &this.in_process {
        GlobalEventBus::publish(WindowsReportMessage::Report(Arc::new(agent.report())));
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
