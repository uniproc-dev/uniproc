use std::time::Duration;

use app_contracts::features::agent_link::{AgentLinkMsg, AgentLinkState, InProcess, StartInProcess};
use app_contracts::features::agents::{
    AgentConnectionState, RunWindowsAgentInProcess, WindowsAgentInProcess, WindowsAgentRuntimeEvent,
};
use guinea::prelude::*;

use super::elevation::{Elevation, RelaunchError};

pub struct Offer;

#[expect(non_upper_case_globals)]
impl Offer {
    pub const InProcessAfter: Duration = Duration::from_secs(5);
}

#[derive(Clone, Debug)]
pub struct OfferInProcessLater;

#[derive(Clone, Debug)]
pub struct InProcessOfferDue;

struct Relaunched(Result<(), RelaunchError>);

pub struct AgentLinkActor {
    ui_port: Push<AgentLinkState>,
    elevation: Elevation,
    elevated: bool,
    in_process: InProcess,
    last: Option<AgentConnectionState>,
}

impl std::fmt::Debug for AgentLinkActor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AgentLinkActor")
            .field("elevated", &self.elevated)
            .field("in_process", &self.in_process)
            .field("last", &self.last)
            .finish()
    }
}

impl AgentLinkActor {
    pub fn new(ui_port: Push<AgentLinkState>, elevation: Elevation) -> Self {
        let elevated = (elevation.elevated)();
        ui_port.send(AgentLinkMsg::Elevated(elevated));
        Self {
            ui_port,
            elevation,
            elevated,
            in_process: InProcess::Off,
            last: None,
        }
    }

    fn set(&mut self, in_process: InProcess) {
        self.in_process = in_process;
        self.ui_port.send(AgentLinkMsg::InProcess(in_process));
    }
}

actor! {
    AgentLinkActor {
        handlers {
            WindowsAgentRuntimeEvent,
            WindowsAgentInProcess,
            StartInProcess,
            Relaunched,
            OfferInProcessLater,
            InProcessOfferDue,
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
    if matches!(this.in_process, InProcess::Starting | InProcess::Elevating | InProcess::Running) {
        return;
    }
    if !this.elevated {
        this.set(InProcess::Elevating);
        let relaunch = this.elevation.relaunch;
        cx.spawn_bg(async move { Relaunched(relaunch().await) });
        return;
    }
    this.set(InProcess::Starting);
    GlobalEventBus::publish(RunWindowsAgentInProcess);
}

#[handler]
fn on_in_process(this: &mut AgentLinkActor, started: WindowsAgentInProcess) {
    this.set(match started {
        WindowsAgentInProcess::Running => InProcess::Running,
        WindowsAgentInProcess::NotElevated => InProcess::NotElevated,
        WindowsAgentInProcess::Failed => InProcess::Failed,
    });
}

#[handler]
fn on_relaunched(this: &mut AgentLinkActor, Relaunched(relaunched): Relaunched) {
    match relaunched {
        Ok(()) => {
            tracing::info!("uniproc restarts as administrator to monitor in process");
            (this.elevation.close)();
        }
        Err(RelaunchError::Refused) => this.set(InProcess::Off),
        Err(RelaunchError::Failed(error)) => {
            tracing::warn!(%error, "uniproc did not restart as administrator");
            this.set(InProcess::Failed);
        }
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
