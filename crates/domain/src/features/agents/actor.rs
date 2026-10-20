use super::backend::{AgentBackend, AgentEvent, InProcessError, Outdated};
use super::connection::*;
use amethystate::Field;
use app_contracts::features::agents::{AgentConnectionState, AgentStateRequest, RunWindowsAgentInProcess};
use guinea::prelude::*;
use futures::stream::{AbortHandle, BoxStream};
use futures::{StreamExt, future::BoxFuture};
use app_contracts::features::agents::{WindowsMachineSample, WindowsProcessEvents, WindowsAgentInProcess, WindowsAgentRuntimeEvent, WslAgentRuntimeEvent, WindowsReportMessage, RemoteScanResult};
use super::actions::WindowsTransport;
use std::fmt::Debug;
use tracing::{info, warn};

#[derive(Clone, Debug)]
pub struct Init;

#[derive(Clone, Debug)]
pub struct Ping;

#[derive(Clone, Debug)]
pub struct StartConnect;

#[derive(Clone, Debug)]
pub struct TryConnectWithDelay {
    delay: std::time::Duration,
    epoch: u64,
}

#[derive(Clone, Debug)]
pub struct RetryTimerElapsed(u64);

#[derive(Clone, Debug)]
pub struct ConnectionLost;

#[derive(Clone, Debug)]
pub struct PingResult {
    generation: u64,
    ms: Option<i32>,
}

#[derive(Clone, Debug)]
pub struct Streamed {
    generation: u64,
    ok: bool,
}

#[derive(Clone, Debug)]
pub struct ReconnectAfter {
    rest: std::time::Duration,
    epoch: u64,
}

#[derive(Clone, Debug)]
struct ReconnectDue(u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Source {
    Service,
    InProcess,
}

enum Refused {
    Failed,
    Outdated,
    InProcess(InProcessError),
}

struct ConnectResult<B: AgentBackend> {
    epoch: u64,
    result: Result<B::Client, Refused>,
}

#[derive(Clone, Copy, Debug)]
enum Feed {
    Report,
    Machine,
    ProcessEvents,
}

impl Feed {
    fn streams<B: AgentBackend>(self) -> bool {
        match self {
            Feed::Report => true,
            Feed::Machine => B::STREAMS_MACHINE,
            Feed::ProcessEvents => B::STREAMS_PROCESS_EVENTS,
        }
    }

    async fn scan<B: AgentBackend>(self, client: &B::Client, cx: &AsyncContext<GenericAgentActor<B>>) -> anyhow::Result<()> {
        match self {
            Feed::Report => B::perform_scan(client, cx).await,
            Feed::Machine => B::perform_machine_scan(client, cx).await,
            Feed::ProcessEvents => B::perform_process_events(client, cx).await,
        }
    }
}

#[derive(Debug)]
pub struct GenericAgentActor<B: AgentBackend> {
    client: Option<B::Client>,
    connection: ConnectionMachine,
    ping_in_flight: bool,
    attempt_secs: Field<u64>,
    update_interval_ms: Field<u64>,
    attempt_started: Option<tokio::time::Instant>,
    connected_at: Option<tokio::time::Instant>,
    source: Source,
    epoch: u64,
    stream: Option<(u64, Vec<AbortHandle>)>,
    generation: u64,
}

impl<B: AgentBackend> GenericAgentActor<B> {
    pub fn new(attempt_secs: Field<u64>, update_interval_ms: Field<u64>) -> Self {
        Self {
            client: None,
            connection: ConnectionMachine::new(),
            ping_in_flight: false,
            attempt_secs,
            update_interval_ms,
            attempt_started: None,
            connected_at: None,
            source: Source::Service,
            epoch: 0,
            stream: None,
            generation: 0,
        }
    }

    fn switch_to(&mut self, source: Source, cx: &AsyncContext<Self>) {
        self.source = source;
        self.epoch += 1;
        self.client = None;
        B::announce(None, cx);
        self.ping_in_flight = false;
        self.close_stream();
        self.connected_at = None;
        self.connection = ConnectionMachine::new();
    }

    fn open_stream(&mut self, cx: AsyncContext<Self>) -> Vec<BoxStream<'static, Streamed>> {
        let Some(client) = self.client.clone() else {
            return Vec::new();
        };
        self.close_stream();
        self.generation += 1;
        let generation = self.generation;
        let mut feeds = Vec::new();
        let mut streams: Vec<BoxStream<'static, Streamed>> = Vec::new();
        for feed in [Feed::Report, Feed::Machine, Feed::ProcessEvents]
            .into_iter()
            .filter(|feed| feed.streams::<B>())
        {
            let updates = futures::stream::unfold(Some((client.clone(), cx.clone())), move |input| async move {
                let (client, cx) = input?;
                match feed.scan::<B>(&client, &cx).await {
                    Ok(()) => Some((true, Some((client, cx)))),
                    Err(err) => {
                        warn!("[{}] The {feed:?} stream ended: {err}", B::NAME);
                        Some((false, None))
                    }
                }
            });
            let (updates, closing) = futures::stream::abortable(updates);
            feeds.push(closing);
            streams.push(Box::pin(updates.map(move |ok| Streamed { generation, ok })));
        }
        self.stream = Some((generation, feeds));
        streams
    }

    fn close_stream(&mut self) {
        if let Some((_, feeds)) = self.stream.take() {
            for feed in feeds {
                feed.abort();
            }
        }
    }

    fn attempt_window(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.attempt_secs.get())
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

    fn publish_state(&self, latency_ms: Option<i32>, cx: &AsyncContext<Self>) {
        let state = self.connection.state();
        Self::publish(B::create_runtime_event(state, latency_ms).into(), cx);

        if !matches!(state, AgentConnectionState::Connected) {
            Self::publish(B::scan_unavailable(state).into(), cx);
        }
    }

    fn publish(event: AgentEvent, cx: &AsyncContext<Self>) {
        match event {
            AgentEvent::WindowsState(event) => cx.publish(event),
            AgentEvent::WslState(event) => cx.publish(event),
            AgentEvent::WindowsReport(event) => cx.publish(event),
            AgentEvent::WslReport(event) => cx.publish(event),
        }
    }

    fn connect(&mut self) -> BoxFuture<'static, ConnectResult<B>> {
        self.attempt_started = Some(tokio::time::Instant::now());
        let timeout = self.attempt_window();
        let update_interval_ms = self.update_interval_ms.clone();
        let epoch = self.epoch;
        match self.source {
            Source::Service => Box::pin(async move {
                let result = match B::connect(timeout, update_interval_ms).await {
                    Ok(client) => Ok(client),
                    Err(err) if err.downcast_ref::<Outdated>().is_some() => {
                        warn!(agent = B::NAME, error = %err, "connect refused");
                        Err(Refused::Outdated)
                    }
                    Err(err) => {
                        warn!(agent = B::NAME, error = %err, "connect failed");
                        Err(Refused::Failed)
                    }
                };
                ConnectResult::<B> { epoch, result }
            }),
            Source::InProcess => Box::pin(async move {
                let result = B::connect_in_process(update_interval_ms).await.map_err(Refused::InProcess);
                ConnectResult::<B> { epoch, result }
            }),
        }
    }
}

actor! {
    GenericAgentActor<B: AgentBackend> {
        handlers {
            Init => { send StartConnect }
            StartConnect => { task ConnectResult<B> }
            ConnectResult<B> => { send Ping, send StartConnect loop, send TryConnectWithDelay, task Streamed }
            Ping => { task PingResult }
            PingResult => { send ConnectionLost }
            Streamed => { send ConnectionLost }
            TryConnectWithDelay => { task RetryTimerElapsed }
            RetryTimerElapsed => { task ConnectResult<B> loop }
            ReconnectAfter => { task ReconnectDue }
            ReconnectDue => { send StartConnect loop }
            ConnectionLost => { send StartConnect loop, send ReconnectAfter }
        }
        subscribes {
            AgentStateRequest,
            RunWindowsAgentInProcess => { send StartConnect }
        }
        publishes { WindowsAgentRuntimeEvent, WslAgentRuntimeEvent, WindowsReportMessage, RemoteScanResult, WindowsMachineSample, WindowsProcessEvents, WindowsTransport, WindowsAgentInProcess }
    }
}

#[handler]
fn init<B: AgentBackend>(In(this, _msg, cx): In<GenericAgentActor<B>, Init>) {
    info!("[{}] Actor init", B::NAME);
    this.publish_state(None, &cx.async_ctx());
    cx.send(StartConnect);
}

#[handler]
fn start_connect<B: AgentBackend>(In(this, _msg, cx): In<GenericAgentActor<B>, StartConnect>) {
    if let Some(t) = this.apply(ConnectionEvent::BeginConnect)
        && t.to == AgentConnectionState::Connecting
    {
        this.publish_state(None, &cx.async_ctx());
        cx.task().spawn(this.connect());
    }
}

#[handler]
fn on_connect_result<B: AgentBackend>(In(this, ConnectResult { epoch, result }, cx): In<GenericAgentActor<B>, ConnectResult<B>>) {
    if epoch != this.epoch {
        return;
    }
    let event = match result {
        Ok(client) => {
            if this.apply(ConnectionEvent::ConnectSucceeded).is_some() {
                info!("[{}] Connected ({:?})", B::NAME, this.source);
                if this.source == Source::InProcess {
                    B::in_process_started(Ok(()), &cx.async_ctx());
                }
                B::announce(Some(&client), &cx.async_ctx());
                this.client = Some(client);
                this.connected_at = Some(tokio::time::Instant::now());
                this.ping_in_flight = false;
                this.publish_state(None, &cx.async_ctx());
                cx.send(Ping);
                for (lane, stream) in this.open_stream(cx.async_ctx()).into_iter().enumerate() {
                    cx.task().latest().keyed(lane).pour(stream, std::convert::identity);
                }
            }
            return;
        }
        Err(Refused::InProcess(error)) => {
            warn!(agent = B::NAME, %error, "back to the service");
            B::in_process_started(Err(&error), &cx.async_ctx());
            this.switch_to(Source::Service, &cx.async_ctx());
            cx.send(StartConnect);
            return;
        }
        Err(Refused::Failed) => ConnectionEvent::ConnectFailed,
        Err(Refused::Outdated) => ConnectionEvent::ConnectOutdated,
    };
    if let Some(t) = this.apply(event) {
        this.client = None;
        B::announce(None, &cx.async_ctx());
        this.publish_state(None, &cx.async_ctx());
        if t.effect == TransitionEffect::ScheduleRetry {
            let spent = this
                .attempt_started
                .map_or(std::time::Duration::ZERO, |started| started.elapsed());
            cx.send(TryConnectWithDelay {
                delay: retry_in(this.attempt_window(), spent),
                epoch: this.epoch,
            });
        }
    }
}

#[handler]
fn ping<B: AgentBackend>(In(this, _msg, cx): In<GenericAgentActor<B>, Ping>) {
    if !matches!(this.connection.state(), AgentConnectionState::Connected) || this.ping_in_flight {
        return;
    }
    let Some(client) = this.client.clone() else {
        return;
    };
    this.ping_in_flight = true;
    let generation = this.generation;
    cx.task().spawn(async move {
        let ms = match B::ping(&client).await {
            Ok(ms) => Some(ms),
            Err(err) => {
                warn!("[{}] Ping failed: {err}", B::NAME);
                None
            }
        };
        PingResult { generation, ms }
    });
}

#[handler]
fn on_ping_result<B: AgentBackend>(In(this, PingResult { generation, ms }, cx): In<GenericAgentActor<B>, PingResult>) {
    if !this.ping_in_flight || generation != this.generation {
        return;
    }
    this.ping_in_flight = false;
    match ms {
        Some(ms) => this.publish_state(Some(ms), &cx.async_ctx()),
        None => cx.send(ConnectionLost),
    }
}

#[handler]
fn on_streamed<B: AgentBackend>(In(this, Streamed { generation, ok }, cx): In<GenericAgentActor<B>, Streamed>) {
    if ok || this.stream.as_ref().is_none_or(|(current, _)| *current != generation) {
        return;
    }
    this.close_stream();
    cx.send(ConnectionLost);
}

#[handler]
fn schedule_retry<B: AgentBackend>(In(_, TryConnectWithDelay { delay, epoch }, cx): In<GenericAgentActor<B>, TryConnectWithDelay>) {
    cx.task().spawn(async move {
        tokio::time::sleep(delay).await;
        RetryTimerElapsed(epoch)
    });
}

#[handler]
fn on_retry_elapsed<B: AgentBackend>(In(this, RetryTimerElapsed(epoch), cx): In<GenericAgentActor<B>, RetryTimerElapsed>) {
    if epoch != this.epoch {
        return;
    }
    if let Some(t) = this.apply(ConnectionEvent::RetryDelayElapsed)
        && t.to == AgentConnectionState::Connecting
    {
        this.publish_state(None, &cx.async_ctx());
        cx.task().spawn(this.connect());
    }
}

#[handler]
fn reconnect_after<B: AgentBackend>(In(_, ReconnectAfter { rest, epoch }, cx): In<GenericAgentActor<B>, ReconnectAfter>) {
    cx.task().spawn(async move {
        tokio::time::sleep(rest).await;
        ReconnectDue(epoch)
    });
}

#[handler]
fn reconnect_due<B: AgentBackend>(In(this, ReconnectDue(epoch), cx): In<GenericAgentActor<B>, ReconnectDue>) {
    if epoch == this.epoch {
        cx.send(StartConnect);
    }
}

#[handler]
fn on_connection_lost<B: AgentBackend>(In(this, _msg, cx): In<GenericAgentActor<B>, ConnectionLost>) {
    if this.apply(ConnectionEvent::ConnectionLost).is_none() {
        return;
    }
    warn!("[{}] Connection lost", B::NAME);
    this.client = None;
    B::announce(None, &cx.async_ctx());
    this.ping_in_flight = false;
    this.close_stream();
    this.publish_state(None, &cx.async_ctx());
    let lasted = this
        .connected_at
        .take()
        .map_or(std::time::Duration::MAX, |at| at.elapsed());
    match retry_in(this.attempt_window(), lasted) {
        std::time::Duration::ZERO => cx.send(StartConnect),
        rest => {
            warn!("[{}] the connection lasted {lasted:?}; reconnecting in {rest:?}", B::NAME);
            cx.send(ReconnectAfter { rest, epoch: this.epoch });
        }
    }
}

#[handler]
fn on_state_request<B: AgentBackend>(In(this, _msg, cx): In<GenericAgentActor<B>, AgentStateRequest>) {
    this.publish_state(None, &cx.async_ctx());
}

#[handler]
fn run_in_process<B: AgentBackend>(In(this, _msg, cx): In<GenericAgentActor<B>, RunWindowsAgentInProcess>) {
    if !B::SUPPORTS_IN_PROCESS || this.source == Source::InProcess {
        return;
    }
    info!("[{}] monitoring in process instead of through the service", B::NAME);
    this.switch_to(Source::InProcess, &cx.async_ctx());
    cx.send(StartConnect);
}
