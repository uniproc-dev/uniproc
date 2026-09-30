use crate::features::agents::actor::{GenericAgentActor, Init, Ping};
use crate::features::agents::backend::AgentBackend;
use crate::features::agents::decode;
use crate::features::agents::linux_report::{LinuxReports, Update};
use crate::features::agents::rpc::{RpcHandle, RpcService};
use crate::features::agents::settings::AgentSettings;
use crate::features::settings::settings::GeneralSettings;
use anyhow::{anyhow, bail};
use app_contracts::features::agents::{
    AgentConnectionState, AgentStateRequest, LinuxReport, RemoteScan, RemoteScanResult, WslAgentRuntimeEvent,
};
use app_contracts::features::settings::UpdateInterval;
use futures::StreamExt;
use futures::channel::{mpsc, oneshot};
use guinea::prelude::*;
use guinea::ratelimit;
use ogurpchik::auth::handshake::{HandshakeMode, Protocol, authenticate_client};
use ogurpchik::endpoint::Endpoint;
use ogurpchik::rpc::{RpcSession, Side, spawn_session};
use std::io::Write;
use std::os::windows::process::CommandExt;
use std::process::{Child, Command, Stdio};
use std::rc::Rc;
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};
use tracing::{instrument, warn};
use uniproc_protocol::linux_capnp::{self, agent_listener, linux_agent};
use uniproc_protocol::{LINUX_PROTOCOL, WSL_AGENT_VSOCK_PORT};
use uuid::Uuid;

const SCHEMA_ID: &str = "wsl";

const LINUX: Protocol = Protocol::new(
    LINUX_PROTOCOL.id,
    LINUX_PROTOCOL.major,
    LINUX_PROTOCOL.minor,
    LINUX_PROTOCOL.patch,
);

struct HostStub;
impl linux_agent::Server for HostStub {}

pub struct WslSession {
    rpc: RpcSession<linux_agent::Client>,
    child: Child,
}

impl WslSession {
    fn remote(&self) -> &linux_agent::Client {
        self.rpc.remote()
    }
}

impl Drop for WslSession {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

fn wsl() -> Command {
    let mut command = Command::new("wsl.exe");
    command.creation_flags(CREATE_NO_WINDOW);
    command
}

fn launch_agent(distro: &str, agent_path: &str, secret: &str) -> anyhow::Result<Child> {
    let process_name = agent_path.rsplit(['/', '\\']).next().unwrap_or(agent_path);
    let _ = wsl()
        .args(["-d", distro, "-u", "root", "--", "pkill", "-x", process_name])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    let mut child = wsl()
        .args(["-d", distro, "-u", "root", "--", agent_path])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| anyhow!("failed to launch the WSL agent via wsl.exe: {e}"))?;

    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| anyhow!("wsl.exe gave us no stdin to write the secret to"))?;
    stdin.write_all(secret.as_bytes())?;
    drop(stdin);

    Ok(child)
}

fn generate_secret() -> String {
    let mut secret = String::with_capacity(64);
    for byte in Uuid::new_v4().as_bytes().iter().chain(Uuid::new_v4().as_bytes()) {
        secret.push_str(&format!("{byte:02x}"));
    }
    secret
}

enum Pushed {
    Update(Update),
    Unreadable(anyhow::Error),
    Ended,
}

pub struct Delivery {
    pushed: Pushed,
    taken: oneshot::Sender<()>,
}

pub enum WslRequest {
    Ping,
    Watch {
        interval: Duration,
        updates: mpsc::UnboundedSender<Delivery>,
        released: oneshot::Receiver<()>,
    },
}

pub enum WslReply {
    Pong,
    Watching,
}

struct Listener {
    updates: mpsc::UnboundedSender<Delivery>,
}

impl Listener {
    async fn deliver(&self, pushed: Pushed) -> Result<(), capnp::Error> {
        let gone = || capnp::Error::failed("nobody watches the WSL agent any more".into());
        let (taken, answer) = oneshot::channel();
        self.updates.unbounded_send(Delivery { pushed, taken }).map_err(|_| gone())?;
        answer.await.map_err(|_| gone())
    }
}

impl agent_listener::Server for Listener {
    async fn update(
        self: Rc<Self>,
        params: agent_listener::UpdateParams,
        _: agent_listener::UpdateResults,
    ) -> Result<(), capnp::Error> {
        let pushed = match params.get().map_err(anyhow::Error::from).and_then(decode::update) {
            Ok(update) => Pushed::Update(update),
            Err(err) => Pushed::Unreadable(err),
        };
        self.deliver(pushed).await
    }

    async fn ended(
        self: Rc<Self>,
        _: agent_listener::EndedParams,
        _: agent_listener::EndedResults,
    ) -> Result<(), capnp::Error> {
        let _ = self.deliver(Pushed::Ended).await;
        Ok(())
    }
}

fn spec(interval: Duration, mut out: linux_capnp::metric_spec::Builder<'_>) {
    use linux_capnp::{MachineMetric, ProcessMetric};
    const PROCESSES: [ProcessMetric; 8] = [
        ProcessMetric::CpuRunTime,
        ProcessMetric::ResidentSet,
        ProcessMetric::DiskReadBytes,
        ProcessMetric::DiskWriteBytes,
        ProcessMetric::PipeReadBytes,
        ProcessMetric::PipeWriteBytes,
        ProcessMetric::SendfileBytes,
        ProcessMetric::Transports,
    ];
    const MACHINE: [MachineMetric; 4] = [
        MachineMetric::Cpu,
        MachineMetric::Memory,
        MachineMetric::Disk,
        MachineMetric::Network,
    ];
    out.set_interval_ms(interval.as_millis().min(u128::from(u32::MAX)) as u32);
    let mut processes = out.reborrow().init_processes(PROCESSES.len() as u32);
    for (i, metric) in PROCESSES.into_iter().enumerate() {
        processes.set(i as u32, metric);
    }
    let mut machine = out.init_machine(MACHINE.len() as u32);
    for (i, metric) in MACHINE.into_iter().enumerate() {
        machine.set(i as u32, metric);
    }
}

pub struct WslRpc;

static LAUNCH: OnceLock<(String, String)> = OnceLock::new();

pub fn set_launch_config(distro: impl Into<String>, agent_path: impl Into<String>) {
    let _ = LAUNCH.set((distro.into(), agent_path.into()));
}

impl RpcService for WslRpc {
    type Session = Rc<WslSession>;
    type Request = WslRequest;
    type Reply = WslReply;

    const NAME: &'static str = "WSL";

    async fn connect(timeout_secs: u64) -> anyhow::Result<Self::Session> {
        let (distro, agent_path) = LAUNCH
            .get()
            .ok_or_else(|| anyhow!("WSL launch settings were never published"))?;

        let secret = generate_secret();
        let child = launch_agent(distro, agent_path, &secret)?;

        let endpoint =
            Endpoint::vsock_to_wsl(WSL_AGENT_VSOCK_PORT).map_err(|e| anyhow!("{e:#}"))?;

        let mut conn = endpoint
            .connect_ready(Duration::from_secs(timeout_secs))
            .await
            .map_err(|e| anyhow!("{e:#}"))?;

        authenticate_client(
            &mut conn,
            &HandshakeMode::hmac(secret.into_bytes()),
            LINUX,
        )
            .await
            .map_err(|e| anyhow!("{e:#}"))?;

        Ok(Rc::new(WslSession {
            rpc: spawn_session::<linux_agent::Client, _>(conn, Side::Client, HostStub),
            child,
        }))
    }

    async fn dispatch(session: Self::Session, request: Self::Request) -> anyhow::Result<Self::Reply> {
        let client = session.remote();

        match request {
            WslRequest::Ping => {
                client.ping_request().send().promise.await?;
                Ok(WslReply::Pong)
            }
            WslRequest::Watch {
                interval,
                updates,
                released,
            } => {
                let mut request = client.watch_request();
                spec(interval, request.get().init_spec());
                request.get().set_listener(capnp_rpc::new_client(Listener { updates }));
                let handle = request.send().promise.await?.get()?.get_handle()?;
                compio::runtime::spawn(async move {
                    let _handle = handle;
                    let _ = released.await;
                })
                .detach();
                Ok(WslReply::Watching)
            }
        }
    }
}

struct LinuxWatch {
    updates: mpsc::UnboundedReceiver<Delivery>,
    _release: oneshot::Sender<()>,
}

#[derive(Default)]
struct Feed {
    watch: Option<(Duration, LinuxWatch)>,
    reports: LinuxReports,
}

#[derive(Clone)]
pub struct WslClient {
    rpc: RpcHandle<WslRpc>,
    feed: Arc<tokio::sync::Mutex<Feed>>,
    interval: Arc<dyn Fn() -> Duration + Send + Sync>,
}

impl std::fmt::Debug for WslClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WslClient").field("rpc", &self.rpc).finish_non_exhaustive()
    }
}

impl WslClient {
    pub async fn connect(
        timeout_secs: u64,
        interval: impl Fn() -> Duration + Send + Sync + 'static,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            rpc: RpcHandle::connect(timeout_secs).await?,
            feed: Arc::default(),
            interval: Arc::new(interval),
        })
    }

    async fn watch(&self, interval: Duration) -> anyhow::Result<LinuxWatch> {
        let (updates, rx) = mpsc::unbounded();
        let (release, released) = oneshot::channel();
        match self
            .rpc
            .call(WslRequest::Watch {
                interval,
                updates,
                released,
            })
            .await?
        {
            WslReply::Watching => Ok(LinuxWatch {
                updates: rx,
                _release: release,
            }),
            _ => bail!("WSL agent answered watch with something else"),
        }
    }

    pub async fn report(&self) -> anyhow::Result<LinuxReport> {
        let mut feed = self.feed.lock().await;
        loop {
            let interval = (self.interval)();
            if feed.watch.as_ref().is_none_or(|(held, _)| *held != interval) {
                feed.watch = None;
                feed.reports = LinuxReports::default();
                feed.watch = Some((interval, self.watch(interval).await?));
            }
            let Some((_, watch)) = feed.watch.as_mut() else {
                continue;
            };
            let Some(delivery) = watch.updates.next().await else {
                feed.watch = None;
                bail!("the WSL agent watch ended");
            };
            let _ = delivery.taken.send(());
            match delivery.pushed {
                Pushed::Update(update) => match feed.reports.apply(update) {
                    Ok(report) => return Ok(report),
                    Err(resync) => {
                        warn!("[WSL] {resync}; watching again");
                        feed.watch = None;
                    }
                },
                Pushed::Unreadable(err) => {
                    warn!("[WSL] an update this client cannot read: {err:#}; watching again");
                    feed.watch = None;
                }
                Pushed::Ended => {
                    feed.watch = None;
                    bail!("the WSL agent stopped");
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct WslBackend;

impl AgentBackend for WslBackend {
    type Client = WslClient;
    type RuntimeEvent = WslAgentRuntimeEvent;
    type ScanMessage = RemoteScanResult;
    const NAME: &'static str = "WSL";
    const STREAMS: bool = true;

    async fn connect(timeout: u64) -> anyhow::Result<Self::Client> {
        let interval = GeneralSettings::new()?.update_interval_ms();
        WslClient::connect(timeout, move || UpdateInterval::clamp(interval.get())).await
    }

    async fn ping(client: &Self::Client) -> anyhow::Result<i32> {
        let start = Instant::now();
        match client.rpc.call(WslRequest::Ping).await? {
            WslReply::Pong => Ok(start.elapsed().as_millis() as i32),
            _ => bail!("WSL agent answered a ping with something else"),
        }
    }

    #[instrument(skip(client), level = "debug", fields(target = "wsl"), err)]
    async fn perform_scan(client: &Self::Client) -> anyhow::Result<()> {
        let report = client.report().await?;
        GlobalEventBus::publish(RemoteScanResult::Scan(RemoteScan {
            schema_id: SCHEMA_ID,
            processes: report.processes,
            machine: report.machine,
            environments: report.environments,
            docker_containers: report.docker_containers,
        }));
        ratelimit!(3600, info!("Report published to event bus"));
        Ok(())
    }

    fn create_runtime_event(state: AgentConnectionState, latency: Option<i32>) -> Self::RuntimeEvent {
        WslAgentRuntimeEvent { state, latency_ms: latency }
    }

    fn scan_unavailable(state: AgentConnectionState) -> Self::ScanMessage {
        RemoteScanResult::Unavailable(state)
    }
}

pub fn wsl_agent_feature(app: &mut FeatureBuilder) -> anyhow::Result<()> {
    let settings = AgentSettings::new()?;
    let ping_interval = settings.ping_interval_ms();

    set_launch_config(settings.wsl_distro().get(), settings.wsl_agent_path().get());

    let addr = app.spawn(GenericAgentActor::<WslBackend>::new(
        settings.wsl_connect_timeout_secs(),
    ));

    app.every(
        Period::varying(move || Duration::from_millis(ping_interval.get())),
        &addr,
        || Ping,
    )
    .named("wsl-agent-ping");

    addr.subscribe_on::<AgentStateRequest>(Bus::Global);
    addr.send(Init);

    Ok(())
}
