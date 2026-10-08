use std::sync::Arc;
use std::time::Duration;

use app_contracts::features::agents::{ActionOutcome, WindowsAction, WindowsActionRequest};
use futures::future::BoxFuture;
use guinea::core::actor::event_bus::{AsyncBus, RpcRequest};
use guinea::prelude::*;

pub trait ActsOnWindows: Send + Sync + 'static {
    fn act(&self, action: WindowsAction) -> BoxFuture<'static, u32>;
}

#[derive(Clone, Event)]
pub enum WindowsTransport {
    Connected(Arc<dyn ActsOnWindows>),
    Lost,
}

impl std::fmt::Debug for WindowsTransport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Connected(_) => "Connected",
            Self::Lost => "Lost",
        })
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
    transport: Option<Arc<dyn ActsOnWindows>>,
}

impl std::fmt::Debug for WindowsActions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WindowsActions")
            .field("connected", &self.transport.is_some())
            .finish()
    }
}

actor! {
    WindowsActions {
        handlers { WindowsTransport, RpcRequest<WindowsActionRequest> }
    }
}

#[handler]
fn on_transport(this: &mut WindowsActions, transport: WindowsTransport) {
    this.transport = match transport {
        WindowsTransport::Connected(agent) => Some(agent),
        WindowsTransport::Lost => None,
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
