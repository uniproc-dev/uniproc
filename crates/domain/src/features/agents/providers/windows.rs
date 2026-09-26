use crate::features::agents::actor::{GenericAgentActor, Init, Ping};
use crate::features::agents::backend::AgentBackend;
use crate::features::agents::decode;
use crate::features::agents::rpc::{RpcHandle, RpcService};
use crate::features::agents::settings::AgentSettings;
use anyhow::{anyhow, bail};
use app_contracts::features::agents::{
    AgentConnectionState, AgentStateRequest, ScanTick, WindowsAction, WindowsActionRequest,
    WindowsAgentInProcess, WindowsAgentRuntimeEvent, WindowsReport, WindowsReportMessage,
};
use guinea::prelude::*;
use guinea::ratelimit;
use ogurpchik::auth::handshake::{HandshakeMode, SchemaId, authenticate_client};
use ogurpchik::endpoint::Endpoint;
use ogurpchik::rpc::{RpcSession, Side, spawn_session};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::{Duration, Instant};
use tracing::instrument;
use uniproc_protocol::meta_capnp::ResponseStatus;
use uniproc_protocol::windows_capnp::windows_agent;

use uniproc_protocol::{APP_NAME, WINDOWS_AGENT_SERVICE, WINDOWS_SCHEMA_ID};

pub const AGENT_SERVICE_DISPLAY_NAME: &str = "Uniproc Process Monitor";

struct HostStub;
impl windows_agent::Server for HostStub {}

fn agent_service() -> String {
    #[cfg(debug_assertions)]
    if let Some(service) = std::env::var_os("UNIPROC_AGENT_PIPE") {
        return service.to_string_lossy().into_owned();
    }
    WINDOWS_AGENT_SERVICE.to_string()
}

pub enum WindowsRequest {
    Ping,
    Scan,
    Action(WindowsAction),
}

pub enum WindowsReply {
    Pong,
    Report(Option<WindowsReport>),
    Code(u32),
}

const JOIN_ATTEMPTS: usize = 3;

struct Tagged<T> {
    etag: u64,
    value: T,
}

#[derive(Default)]
struct Cache {
    services: Option<Tagged<Vec<app_contracts::features::agents::WindowsServiceStats>>>,
    processes: Option<Tagged<decode::Processes>>,
}

impl Cache {
    fn services_etag(&self) -> u64 {
        self.services.as_ref().map_or(0, |s| s.etag)
    }

    fn processes_etag(&self) -> u64 {
        self.processes.as_ref().map_or(0, |p| p.etag)
    }
}

pub struct WindowsSession {
    rpc: RpcSession<windows_agent::Client>,
    cache: RefCell<Cache>,
    nonce: Cell<u64>,
}

impl WindowsSession {
    pub fn remote(&self) -> &windows_agent::Client {
        self.rpc.remote()
    }
}

fn not_modified(meta: uniproc_protocol::meta_capnp::response_meta::Reader<'_>) -> bool {
    matches!(meta.get_status(), Ok(ResponseStatus::NotModified))
}

async fn fetch_processes(
    client: &windows_agent::Client,
    if_none_match: u64,
) -> anyhow::Result<capnp::capability::Response<windows_agent::get_processes_results::Owned>> {
    let mut request = client.get_processes_request();
    request.get().init_meta().set_if_none_match(if_none_match);
    Ok(request.send().promise.await?)
}

fn apply_processes(
    session: &WindowsSession,
    reply: &capnp::capability::Response<windows_agent::get_processes_results::Owned>,
) -> anyhow::Result<()> {
    let reply = reply.get()?;
    let meta = reply.get_meta()?;
    if not_modified(meta) {
        return Ok(());
    }
    let value = decode::windows_processes(reply.get_processes()?)?;
    session.cache.borrow_mut().processes = Some(Tagged {
        etag: meta.get_etag(),
        value,
    });
    Ok(())
}

fn apply_services(
    session: &WindowsSession,
    reply: &capnp::capability::Response<windows_agent::get_services_results::Owned>,
) -> anyhow::Result<()> {
    let reply = reply.get()?;
    let meta = reply.get_meta()?;
    if not_modified(meta) {
        return Ok(());
    }
    let value = decode::windows_services(reply.get_services()?)?;
    session.cache.borrow_mut().services = Some(Tagged {
        etag: meta.get_etag(),
        value,
    });
    Ok(())
}

async fn scan(session: &WindowsSession) -> anyhow::Result<Option<WindowsReport>> {
    let client = session.remote();
    let (services_etag, processes_etag) = {
        let cache = session.cache.borrow();
        (cache.services_etag(), cache.processes_etag())
    };

    let machine = client.get_machine_request().send().promise;
    let mut services = client.get_services_request();
    services.get().init_meta().set_if_none_match(services_etag);
    let services = services.send().promise;
    let processes = fetch_processes(client, processes_etag);
    let metrics = client.get_process_metrics_request().send().promise;
    let (machine, services, processes, metrics) = futures::join!(machine, services, processes, metrics);

    let machine = decode::windows_machine(machine?.get()?.get_machine()?)?;
    apply_services(session, &services?)?;
    apply_processes(session, &processes?)?;

    let mut metrics = metrics?;
    for _ in 0..JOIN_ATTEMPTS {
        let wanted = metrics.get()?.get_processes_etag();
        let held = session.cache.borrow().processes_etag();
        if held == wanted {
            break;
        }
        apply_processes(session, &fetch_processes(client, held).await?)?;
        if session.cache.borrow().processes_etag() == wanted {
            break;
        }
        metrics = client.get_process_metrics_request().send().promise.await?;
    }

    let metrics = metrics.get()?;
    let cache = session.cache.borrow();
    let Some(processes) = cache
        .processes
        .as_ref()
        .filter(|p| p.etag == metrics.get_processes_etag())
    else {
        tracing::debug!("process list kept moving under the metrics, skipping this scan");
        return Ok(None);
    };

    Ok(Some(WindowsReport {
        machine,
        processes: decode::join_metrics(&processes.value, metrics.get_metrics()?),
        services: cache
            .services
            .as_ref()
            .map(|s| s.value.clone())
            .unwrap_or_default(),
    }))
}

pub struct WindowsRpc;

impl RpcService for WindowsRpc {
    type Session = Rc<WindowsSession>;
    type Request = WindowsRequest;
    type Reply = WindowsReply;

    const NAME: &'static str = "Windows";

    async fn connect(timeout_secs: u64) -> anyhow::Result<Self::Session> {
        let endpoint =
            Endpoint::for_service(APP_NAME, &agent_service()).map_err(|e| anyhow!("{e:#}"))?;

        let mut conn = endpoint
            .connect_ready(Duration::from_secs(timeout_secs))
            .await
            .map_err(|e| anyhow!("{e:#}"))?;

        authenticate_client(
            &mut conn,
            &HandshakeMode::version_only(),
            SchemaId(WINDOWS_SCHEMA_ID),
        )
            .await
            .map_err(|e| anyhow!("{e:#}"))?;

        Ok(Rc::new(WindowsSession {
            rpc: spawn_session::<windows_agent::Client, _>(conn, Side::Client, HostStub),
            cache: RefCell::new(Cache::default()),
            nonce: Cell::new(0),
        }))
    }

    async fn dispatch(session: Self::Session, request: Self::Request) -> anyhow::Result<Self::Reply> {
        match request {
            WindowsRequest::Ping => {
                let nonce = session.nonce.get().wrapping_add(1);
                session.nonce.set(nonce);
                let mut request = session.remote().ping_request();
                request.get().set_nonce(nonce);
                let echoed = request.send().promise.await?.get()?.get_nonce();
                if echoed != nonce {
                    bail!("agent echoed nonce {echoed} to ping {nonce}");
                }
                Ok(WindowsReply::Pong)
            }
            WindowsRequest::Scan => Ok(WindowsReply::Report(scan(&session).await?)),
            WindowsRequest::Action(action) => {
                let code = perform_action(session.remote(), action).await?;
                Ok(WindowsReply::Code(code))
            }
        }
    }
}

async fn perform_action(client: &windows_agent::Client, action: WindowsAction) -> anyhow::Result<u32> {
    macro_rules! by_pid {
        ($request:ident, $pid:expr) => {{
            let mut req = client.$request();
            req.get().set_pid($pid);
            req.send().promise.await?.get()?.get_code()
        }};
    }
    macro_rules! by_name {
        ($request:ident, $name:expr) => {{
            let mut req = client.$request();
            req.get().set_name(&$name);
            req.send().promise.await?.get()?.get_code()
        }};
    }

    let code = match action {
        WindowsAction::Kill { pid } => by_pid!(kill_request, pid),
        WindowsAction::Suspend { pid } => by_pid!(suspend_request, pid),
        WindowsAction::Resume { pid } => by_pid!(resume_request, pid),
        WindowsAction::SetPriority { pid, priority } => {
            let mut req = client.set_priority_request();
            req.get().set_pid(pid);
            req.get().set_priority(decode::priority(priority));
            req.send().promise.await?.get()?.get_code()
        }
        WindowsAction::SetAffinity { pid, mask } => {
            let mut req = client.set_affinity_request();
            req.get().set_pid(pid);
            req.get().set_mask(mask);
            req.send().promise.await?.get()?.get_code()
        }
        WindowsAction::ServiceStart { name } => by_name!(service_start_request, name),
        WindowsAction::ServiceStop { name } => by_name!(service_stop_request, name),
        WindowsAction::ServicePause { name } => by_name!(service_pause_request, name),
        WindowsAction::ServiceResume { name } => by_name!(service_resume_request, name),
        WindowsAction::ServiceRestart { name } => by_name!(service_restart_request, name),
    };

    Ok(code)
}

#[derive(Debug)]
pub struct WindowsBackend;

impl AgentBackend for WindowsBackend {
    type Client = RpcHandle<WindowsRpc>;
    type RuntimeEvent = WindowsAgentRuntimeEvent;
    type ScanMessage = WindowsReportMessage;
    const NAME: &'static str = "Windows";

    async fn connect(timeout: u64) -> anyhow::Result<Self::Client> {
        RpcHandle::connect(timeout).await
    }

    async fn ping(client: &Self::Client) -> anyhow::Result<i32> {
        let start = Instant::now();
        match client.call(WindowsRequest::Ping).await? {
            WindowsReply::Pong => Ok(start.elapsed().as_millis() as i32),
            _ => bail!("agent answered a ping with something else"),
        }
    }

    #[instrument(skip(client), level = "debug", err)]
    async fn perform_scan(client: &Self::Client) -> anyhow::Result<()> {
        match client.call(WindowsRequest::Scan).await? {
            WindowsReply::Report(Some(report)) => {
                GlobalEventBus::publish(WindowsReportMessage::Report(std::sync::Arc::new(report)));
                ratelimit!(3600, info!("Report published to event bus"));
                Ok(())
            }
            WindowsReply::Report(None) => Ok(()),
            _ => bail!("agent answered a scan with something else"),
        }
    }

    fn create_runtime_event(state: AgentConnectionState, latency: Option<i32>) -> Self::RuntimeEvent {
        WindowsAgentRuntimeEvent { state, latency_ms: latency }
    }

    fn scan_unavailable(state: AgentConnectionState) -> Self::ScanMessage {
        WindowsReportMessage::Unavailable(state)
    }
}

pub fn windows_agent_feature(app: &mut FeatureBuilder) -> anyhow::Result<()> {
    let settings = AgentSettings::new()?;
    let ping_interval = settings.ping_interval_ms();

    let addr = app.spawn(GenericAgentActor::<WindowsBackend>::new(
        settings.connect_attempt_secs(),
    ));

    app.every(
        Period::varying(move || Duration::from_millis(ping_interval.get())),
        &addr,
        || Ping,
    )
    .named("windows-agent-ping");

    addr.subscribe_on::<ScanTick>(Bus::Global);
    addr.subscribe_on::<WindowsActionRequest>(Bus::Global);
    addr.subscribe_on::<AgentStateRequest>(Bus::Global);
    addr.subscribe_on::<WindowsAgentInProcess>(Bus::Global);
    addr.send(Init);

    Ok(())
}
