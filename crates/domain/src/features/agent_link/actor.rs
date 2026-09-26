use std::time::Duration;

use app_contracts::features::agent_link::{AgentLinkMsg, AgentLinkState, OpenNativeTaskManager};
use app_contracts::features::agents::{AgentConnectionState, WindowsAgentRuntimeEvent};
use guinea::prelude::*;

pub struct Native;

#[expect(non_upper_case_globals)]
impl Native {
    pub const OfferedAfter: Duration = Duration::from_secs(5);
}

#[derive(Clone, Debug)]
pub struct OfferNativeLater;

#[derive(Clone, Debug)]
pub struct NativeOfferDue;

#[derive(Debug)]
pub struct AgentLinkActor {
    ui_port: Push<AgentLinkState>,
    open_native: fn(),
    last: Option<AgentConnectionState>,
}

impl AgentLinkActor {
    pub fn new(ui_port: Push<AgentLinkState>, open_native: fn()) -> Self {
        Self {
            ui_port,
            open_native,
            last: None,
        }
    }
}

actor! {
    AgentLinkActor {
        handlers { WindowsAgentRuntimeEvent, OpenNativeTaskManager, OfferNativeLater, NativeOfferDue }
    }
}

#[handler]
fn on_windows_agent(this: &mut AgentLinkActor, ctx: Context<AgentLinkActor, WindowsAgentRuntimeEvent>) {
    let state = ctx.msg.state;
    if this.last == Some(state) {
        return;
    }
    this.last = Some(state);
    this.ui_port.send(AgentLinkMsg::Windows(state));
}

#[handler]
fn open_native(this: &AgentLinkActor, _ctx: Context<AgentLinkActor, OpenNativeTaskManager>) {
    (this.open_native)();
}

#[handler]
async fn offer_native_later(ctx: AsyncContext<AgentLinkActor>, _msg: OfferNativeLater) {
    let waited = ctx.until_gone(tokio::time::sleep(Native::OfferedAfter)).await;
    if waited.is_some() {
        ctx.send(NativeOfferDue);
    }
}

#[handler]
fn native_offer_due(this: &AgentLinkActor, _ctx: Context<AgentLinkActor, NativeOfferDue>) {
    this.ui_port.send(AgentLinkMsg::OfferNative);
}
