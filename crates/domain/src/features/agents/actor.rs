use super::backend::AgentBackend;
use super::connection::*;
use amethystate::Field;
use app_contracts::features::agents::{AgentConnectionState, AgentStateRequest, ScanTick, WindowsAgentInProcess};
use guinea::prelude::*;
use std::fmt::Debug;
use tracing::{info, warn};

#[derive(Clone, Debug)]
pub struct Init;

#[derive(Clone, Debug)]
pub struct Ping;

#[derive(Clone, Debug)]
pub struct StartConnect;

#[derive(Clone, Debug)]
pub struct TryConnectWithDelay(pub std::time::Duration);

#[derive(Clone, Debug)]
pub struct RetryTimerElapsed;

#[derive(Clone, Debug)]
pub struct ConnectionLost;

#[derive(Clone, Debug)]
pub struct PingResult(pub Option<i32>);

#[derive(Clone, Debug)]
pub struct ScanResult(pub bool);

struct ConnectResult<C>(Option<C>);

#[derive(Debug)]
pub struct GenericAgentActor<B: AgentBackend> {
    client: Option<B::Client>,
    connection: ConnectionMachine,
    ping_in_flight: bool,
    failed_scans: u32,
    attempt_secs: Field<u64>,
    attempt_started: Option<tokio::time::Instant>,
    dormant: bool,
}

impl<B: AgentBackend> GenericAgentActor<B> {
    pub fn new(attempt_secs: Field<u64>) -> Self {
        Self {
            client: None,
            connection: ConnectionMachine::new(),
            ping_in_flight: false,
            failed_scans: 0,
            attempt_secs,
            attempt_started: None,
            dormant: false,
        }
    }

    fn attempt_window(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.attempt_secs.get().max(1))
    }

    fn apply(&mut self, event: ConnectionEvent) -> Option<Transition> {
        match self.connection.apply(event) {
            Ok(t) => Some(t),
            Err(err) => {
                warn!("[{}] FSM invalid: {:?} on {:?}", B::NAME, err.event, err.state);
                None
            }
        }
    }

    fn publish_state(&self, latency_ms: Option<i32>) {
        if self.dormant {
            return;
        }
        let state = self.connection.state();
        GlobalEventBus::publish(B::create_runtime_event(state, latency_ms));

        if !matches!(state, AgentConnectionState::Connected) {
            GlobalEventBus::publish(B::scan_unavailable(state));
        }
    }

    fn spawn_connect(&mut self, cx: &Cx<Self>) {
        self.attempt_started = Some(tokio::time::Instant::now());
        let timeout = self.attempt_secs.get().max(1);
        cx.spawn_bg(async move {
            match B::connect(timeout).await {
                Ok(client) => ConnectResult(Some(client)),
                Err(err) => {
                    warn!(agent = B::NAME, error = %err, "connect failed");
                    ConnectResult(None)
                }
            }
        });
    }
}

actor! {
    GenericAgentActor<B: AgentBackend> {
        handlers {
            Init,
            StartConnect,
            ConnectResult<B::Client>,
            Ping,
            PingResult,
            ScanTick,
            ScanResult,
            TryConnectWithDelay,
            RetryTimerElapsed,
            ConnectionLost,
            AgentStateRequest,
            WindowsAgentInProcess,
        }
    }
}

#[handler]
fn on_in_process<B: AgentBackend>(this: &mut GenericAgentActor<B>, _msg: WindowsAgentInProcess) {
    info!("[{}] the in-process agent took over, going dormant", B::NAME);
    this.dormant = true;
    this.client = None;
    this.ping_in_flight = false;
}

#[handler]
fn on_state_request<B: AgentBackend>(this: &GenericAgentActor<B>, _msg: AgentStateRequest) {
    this.publish_state(None);
}

#[handler]
fn init<B: AgentBackend>(this: &GenericAgentActor<B>, _msg: Init, cx: Cx) {
    info!("[{}] Actor init", B::NAME);
    this.publish_state(None);
    cx.addr().send(StartConnect);
}

#[handler]
fn start_connect<B: AgentBackend>(this: &mut GenericAgentActor<B>, _msg: StartConnect, cx: Cx) {
    if this.dormant {
        return;
    }
    if let Some(t) = this.apply(ConnectionEvent::BeginConnect)
        && t.to == AgentConnectionState::Connecting
    {
        this.publish_state(None);
        this.spawn_connect(&cx.detach());
    }
}

#[handler]
fn on_connect_result<B: AgentBackend>(
    this: &mut GenericAgentActor<B>,
    ConnectResult(client): ConnectResult<B::Client>,
    cx: Cx,
) {
    if this.dormant {
        return;
    }
    let addr = cx.addr();
    match client {
        Some(client) => {
            if this.apply(ConnectionEvent::ConnectSucceeded).is_some() {
                info!("[{}] Connected", B::NAME);
                this.client = Some(client);
                this.ping_in_flight = false;
                this.publish_state(None);
                addr.send(Ping);
            }
        }
        None => {
            if let Some(t) = this.apply(ConnectionEvent::ConnectFailed) {
                this.client = None;
                this.publish_state(None);
                if t.effect == TransitionEffect::ScheduleRetry {
                    let spent = this
                        .attempt_started
                        .map_or(std::time::Duration::ZERO, |started| started.elapsed());
                    addr.send(TryConnectWithDelay(retry_in(this.attempt_window(), spent)));
                }
            }
        }
    }
}

#[handler]
fn ping<B: AgentBackend>(this: &mut GenericAgentActor<B>, _msg: Ping, cx: Cx) {
    if !matches!(this.connection.state(), AgentConnectionState::Connected) || this.ping_in_flight {
        return;
    }
    let Some(client) = this.client.clone() else {
        return;
    };
    this.ping_in_flight = true;
    cx.spawn_bg(async move {
        match B::ping(&client).await {
            Ok(ms) => PingResult(Some(ms)),
            Err(err) => {
                warn!("[{}] Ping failed: {err}", B::NAME);
                PingResult(None)
            }
        }
    });
}

#[handler]
fn on_ping_result<B: AgentBackend>(this: &mut GenericAgentActor<B>, PingResult(ms): PingResult, cx: Cx) {
    if !this.ping_in_flight {
        return;
    }
    this.ping_in_flight = false;
    match ms {
        Some(ms) => this.publish_state(Some(ms)),
        None => cx.addr().send(ConnectionLost),
    }
}

#[handler]
fn perform_scan_tick<B: AgentBackend>(this: &GenericAgentActor<B>, _msg: ScanTick, cx: Cx) {
    if this.dormant || !matches!(this.connection.state(), AgentConnectionState::Connected) {
        return;
    }

    let Some(client) = this.client.clone() else {
        warn!("[{}] client is None (unexpected state)", B::NAME);
        return;
    };

    cx.spawn_bg(async move {
        match B::perform_scan(&client).await {
            Ok(()) => ScanResult(true),
            Err(err) => {
                warn!("[{}] Scan failed: {err}", B::NAME);
                ScanResult(false)
            }
        }
    });
}

#[handler]
fn on_scan_result<B: AgentBackend>(this: &mut GenericAgentActor<B>, ScanResult(ok): ScanResult, cx: Cx) {
    const FAILURES_BEFORE_GIVING_UP: u32 = 3;

    if ok {
        this.failed_scans = 0;
        return;
    }

    this.failed_scans += 1;
    if this.failed_scans >= FAILURES_BEFORE_GIVING_UP {
        warn!("[{}] {} scans in a row failed, treating the agent as gone", B::NAME, this.failed_scans);
        this.failed_scans = 0;
        cx.addr().send(ConnectionLost);
    }
}

#[handler]
async fn schedule_retry<B: AgentBackend>(ctx: AsyncContext<GenericAgentActor<B>>, msg: TryConnectWithDelay) {
    let waited = ctx.until_gone(tokio::time::sleep(msg.0)).await;
    if waited.is_none() {
        return;
    }
    ctx.send(RetryTimerElapsed);
}

#[handler]
fn on_retry_elapsed<B: AgentBackend>(this: &mut GenericAgentActor<B>, _msg: RetryTimerElapsed, cx: Cx) {
    if this.dormant {
        return;
    }
    if let Some(t) = this.apply(ConnectionEvent::RetryDelayElapsed)
        && t.to == AgentConnectionState::Connecting
    {
        this.publish_state(None);
        this.spawn_connect(&cx.detach());
    }
}

#[handler]
fn on_connection_lost<B: AgentBackend>(this: &mut GenericAgentActor<B>, _msg: ConnectionLost, cx: Cx) {
    if this.dormant {
        return;
    }
    if this.apply(ConnectionEvent::ConnectionLost).is_none() {
        return;
    }
    warn!("[{}] Connection lost", B::NAME);
    this.client = None;
    this.ping_in_flight = false;
    this.failed_scans = 0;
    this.publish_state(None);
    cx.addr().send(StartConnect);
}

mod windows {
    use super::*;
    use crate::features::agents::providers::windows::WindowsBackend;
    use app_contracts::features::agents::{WindowsActionRequest, WindowsActionResponse};
    use tracing::error;

    #[handler]
    fn handle_windows_action(
        this: &GenericAgentActor<WindowsBackend>,
        msg: WindowsActionRequest,
        cx: Cx,
    ) {
        if this.dormant {
            return;
        }
        let Some(client) = this.client.clone() else {
            error!("Dropping {:?}: not connected to the agent", msg.action);
            return;
        };

        let correlation_id = msg.correlation_id;
        cx.spawn_bg_detached(async move {
            let code = client.act(msg.action).await;
            GlobalEventBus::publish(WindowsActionResponse::new(correlation_id, code));
        });
    }
}
