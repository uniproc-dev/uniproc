use std::sync::Arc;
use std::time::Duration;

use app_contracts::features::agents::{ActionOutcome, WindowsAction, WindowsActionRequest};
use guinea::core::actor::event_bus::{AsyncBus, RpcRequest};
use guinea::prelude::*;

use crate::features::agent_link::InProcessAgent;
use crate::features::agents::providers::windows::WindowsClient;

#[derive(Clone, Event)]
pub enum WindowsTransport {
    Remote(WindowsClient),
    Local(Arc<dyn InProcessAgent>),
    Lost,
}

impl std::fmt::Debug for WindowsTransport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Remote(_) => "Remote",
            Self::Local(_) => "Local",
            Self::Lost => "Lost",
        })
    }
}

#[derive(Clone)]
enum Transport {
    Remote(WindowsClient),
    Local(Arc<dyn InProcessAgent>),
}

impl Transport {
    async fn act(self, action: WindowsAction) -> u32 {
        match self {
            Self::Remote(client) => client.act(action).await,
            Self::Local(agent) => agent.act(action).await,
        }
    }
}

struct Code;

#[expect(non_upper_case_globals)]
impl Code {
    const Success: u32 = 0;
    const AccessDenied: u32 = 5;
    const InvalidParameter: u32 = 87;
    const Busy: u32 = 170;
    const NoAnswer: u32 = u32::MAX;
}

pub fn outcome(code: u32) -> ActionOutcome {
    match code {
        Code::Success => ActionOutcome::Done,
        Code::AccessDenied => ActionOutcome::Denied,
        Code::InvalidParameter => ActionOutcome::Gone,
        Code::Busy => ActionOutcome::Busy,
        Code::NoAnswer => ActionOutcome::NotConnected,
        other => ActionOutcome::Failed(other),
    }
}

#[derive(Default)]
pub struct WindowsActions {
    transport: Option<Transport>,
}

impl std::fmt::Debug for WindowsActions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let transport = match &self.transport {
            Some(Transport::Remote(_)) => "Remote",
            Some(Transport::Local(_)) => "Local",
            None => "None",
        };
        f.debug_struct("WindowsActions").field("transport", &transport).finish()
    }
}

actor! {
    WindowsActions {
        handlers { WindowsTransport, RpcRequest<WindowsActionRequest> }
    }
}

#[handler]
fn on_transport(this: &mut WindowsActions, transport: WindowsTransport) {
    this.transport = match (this.transport.take(), transport) {
        (Some(Transport::Local(agent)), _) => Some(Transport::Local(agent)),
        (_, WindowsTransport::Remote(client)) => Some(Transport::Remote(client)),
        (_, WindowsTransport::Local(agent)) => Some(Transport::Local(agent)),
        (_, WindowsTransport::Lost) => None,
    };
}

#[handler]
fn act(this: &mut WindowsActions, WindowsActionRequest(action): WindowsActionRequest) -> Reply<ActionOutcome> {
    let Some(transport) = this.transport.clone() else {
        return Reply::now(ActionOutcome::NotConnected);
    };
    Reply::later(async move { outcome(transport.act(action).await) })
}

struct Pace;

#[expect(non_upper_case_globals)]
impl Pace {
    const Answer: Duration = Duration::from_secs(60);
}

pub async fn request(action: WindowsAction) -> ActionOutcome {
    AsyncBus::request(WindowsActionRequest(action), Pace::Answer)
        .await
        .unwrap_or(ActionOutcome::NotConnected)
}

pub fn install(app: &mut FeatureBuilder) {
    let addr = app.spawn(WindowsActions::default());
    addr.subscribe_on::<WindowsTransport>(Bus::Global);
    addr.subscribe_on::<RpcRequest<WindowsActionRequest>>(Bus::Global);
}
