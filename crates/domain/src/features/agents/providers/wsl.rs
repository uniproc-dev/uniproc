use crate::agents_impl::actor::{GenericAgentActor, Init, Ping};
use crate::agents_impl::backend::AgentBackend;
use crate::features::agents::settings::AgentSettings;
use app_contracts::features::agents::{
    AgentConnectionState, RemoteScanResult, ScanTick, WslAgentRuntimeEvent, WslClient,
};
use forsl_core::actor::event_bus::EventBus;
use forsl_core::{actor::addr::Addr, ratelimit};
use forsl::feature::{
    AppFeature, AppFeatureInitContext, ContextActorExt, ContextReactorExt, ContextStoreExt,
};
use macros::app_feature;
use ogurpchik::discovery::register_vm_default;
use ogurpchik::high::node::Node;
use ogurpchik::transport::stream::adapters::vsock::{VsockAddr, VsockTransport};
use rpstate::DefaultStore;
use std::ops::Deref;
use std::time::Instant;
use tracing::{error, instrument, warn};
use uniproc_protocol::{LinuxCodec, LinuxRequest, LinuxResponse, services};

pub struct WslBackend;

impl AgentBackend for WslBackend {
    type Client = WslClient;
    type RuntimeEvent = WslAgentRuntimeEvent;
    const NAME: &'static str = "WSL";

    async fn connect(timeout: u64) -> anyhow::Result<Self::Client> {
        register_vm_default("WSL").ok();
        Node::new()?
            .connect::<LinuxCodec, _>(VsockTransport::client(VsockAddr::SelfManaged))
            .wait_for(services::LINUX_AGENT)
            .timeout(timeout)
            .start()
            .await
    }

    async fn ping(client: &Self::Client) -> anyhow::Result<i32> {
        let start = Instant::now();
        client.call(LinuxRequest::Ping).await?;
        Ok(start.elapsed().as_millis() as i32)
    }

    #[instrument(skip(client), level = "debug", fields(target = "wsl"), err)]
    async fn perform_scan(client: &Self::Client) -> anyhow::Result<()> {
        let resp = client.call(LinuxRequest::GetReport).await?;

        let report = rkyv::deserialize::<LinuxResponse, rkyv::rancor::Error>(*resp.deref())
            .map_err(|e| {
                error!(error = %e, "Failed to deserialize WSL response");
                anyhow::anyhow!("WSL scan deserialization error: {}", e)
            })?;

        if let LinuxResponse::Report(r) = report {
            EventBus::publish(RemoteScanResult {
                schema_id: "wsl",
                processes: r.processes,
                machine: r.machine,
                environments: r.environments,
                docker_containers: r.docker_containers,
            });

            ratelimit!(3600, info!("Report published to event bus"));
        } else {
            warn!(response = ?report, "Unexpected WSL response type â€” strange");
        }

        Ok(())
    }

    fn create_runtime_event(
        state: AgentConnectionState,
        latency: Option<i32>,
    ) -> Self::RuntimeEvent {
        WslAgentRuntimeEvent {
            state: state,
            latency_ms: latency,
        }
    }
}

#[app_feature]
pub fn wsl_agent_feature(ctx: &mut AppFeatureInitContext) -> anyhow::Result<()> {
    let store = ctx.store();
    let settings = AgentSettings::new(&store)?;

    let addr = Addr::new(
        GenericAgentActor::<WslBackend>::new(settings.connect_timeout_secs()),
        ctx.token.clone(),
        ctx.tracker,
    );

    ctx.spawn_heartbeat(&addr, settings.ping_interval_ms().as_signal(), || Ping);

    EventBus::subscribe::<GenericAgentActor<WslBackend>, ScanTick>(addr.clone(), ctx.tracker);
    addr.send(Init);

    Ok(())
}
