use std::time::Duration;

use app_contracts::features::agent_link::{AgentLinkMsg, AgentLinkState, InProcess, StartInProcess};
use app_contracts::features::agents::{
    AgentConnectionState, AgentStateRequest, RunWindowsAgentInProcess, WindowsAgentInProcess, WindowsAgentRuntimeEvent,
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

struct Relaunched(Result<(), RelaunchError>);

pub struct AgentLinkActor {
    push: Push<AgentLinkState>,
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
    pub fn new(push: Push<AgentLinkState>, elevation: Elevation) -> Self {
        let elevated = (elevation.elevated)();
        push.send(AgentLinkMsg::Elevated(elevated));
        Self {
            push,
            elevation,
            elevated,
            in_process: InProcess::Off,
            last: None,
        }
    }

    fn set(&mut self, in_process: InProcess) {
        self.in_process = in_process;
        self.push.send(AgentLinkMsg::InProcess(in_process));
    }
}

actor! {
    AgentLinkActor {
        handlers {
            StartInProcess => { task Relaunched }
            Relaunched,
            OfferInProcessLater,
            AgentStateRequest,
        }
        subscribes { WindowsAgentRuntimeEvent, WindowsAgentInProcess }
        publishes { RunWindowsAgentInProcess, AgentStateRequest }
    }
}

#[handler]
fn request_state(In(_, _, cx): In<AgentLinkActor, AgentStateRequest>) {
    cx.publish(AgentStateRequest);
}

#[handler]
fn on_windows_agent(In(this, WindowsAgentRuntimeEvent { state, .. }, _): In<AgentLinkActor, WindowsAgentRuntimeEvent>) {
    if this.last == Some(state) {
        return;
    }
    this.last = Some(state);
    this.push.send(AgentLinkMsg::Windows(state));
}

#[handler]
fn on_in_process(In(this, started, _): In<AgentLinkActor, WindowsAgentInProcess>) {
    this.set(match started {
        WindowsAgentInProcess::Running => InProcess::Running,
        WindowsAgentInProcess::NotElevated => InProcess::NotElevated,
        WindowsAgentInProcess::Failed => InProcess::Failed,
    });
}

#[handler]
fn start_in_process(In(this, _, cx): In<AgentLinkActor, StartInProcess>) {
    if matches!(this.in_process, InProcess::Starting | InProcess::Elevating | InProcess::Running) {
        return;
    }
    if !this.elevated {
        this.set(InProcess::Elevating);
        let relaunch = this.elevation.relaunch;
        cx.task().spawn(async move { Relaunched(relaunch().await) });
        return;
    }
    this.set(InProcess::Starting);
    cx.publish(RunWindowsAgentInProcess);
}

#[handler]
fn on_relaunched(In(this, Relaunched(relaunched), _): In<AgentLinkActor, Relaunched>) {
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
fn offer_in_process_later(In(this, _, cx): In<AgentLinkActor, OfferInProcessLater>) {
    cx.task().pushing(this.push).spawn(async {
        tokio::time::sleep(Offer::InProcessAfter).await;
        AgentLinkMsg::OfferInProcess
    });
}
