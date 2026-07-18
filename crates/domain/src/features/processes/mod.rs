use forsl::app::Window;

use std::borrow::Cow;

use crate::features::processes::application::actor::*;
use crate::features::processes::domain::table::ProcessTable;
use crate::features::processes::services::metadata::ProcessMetadataService;
use crate::processes_impl::application::process_snapshot_actor::ProcessSnapshotActor;
use crate::processes_impl::settings::ProcessSettings;

use app_contracts::features::agents::ScanTick;
use app_contracts::features::processes::{UiProcessesBindings, UiProcessesPort, UiProcessesPortMsg};
use forsl_core::actor::NoOp;
use forsl_core::actor::addr::Addr;
use forsl_core::actor::event_bus::EventBus;
use context::page_status::RouteStatusRegistry;
use forsl::addr::AddrBuilder;
use forsl::feature::{
    ContextActorExt, ContextReactorExt, ContextStoreExt, FeatureContextState, WindowFeature,
    WindowFeatureInitContext,
};
use macros::window_feature;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

mod application;
mod domain;
mod scanner;
mod services;
mod settings;

#[window_feature]
pub fn process_feature<TWindow, P>(
    ctx: &mut WindowFeatureInitContext<TWindow>,
    ui_port: P,
) -> anyhow::Result<()>
where
    TWindow: Window,
    P: UiProcessesPort + UiProcessesBindings + Clone + 'static,
{
    let store = ctx.store();

    let settings = ProcessSettings::new_with(&store)?;

    let scan_interval_ms = settings.scan_interval_ms();

    let process_actor = ProcessActor {
        table: ProcessTable::new(settings.clone())?,
        metadata: ProcessMetadataService,
        route_status: ctx.shared.get::<RouteStatusRegistry>().unwrap(),
        is_active: true,
        active_context_key: Cow::Borrowed("host"),
        is_grouped: false,
        ui_port: ui_port.clone(),
        has_snapshot_data: false,
        ctx: FeatureContextState::new(ctx.window_id, "processes.list"),
    };

    let addr = ctx.actor_builder(process_actor).ui_bind(&ui_port);

    let snapshot_actor = ProcessSnapshotActor {
        snapshots: HashMap::new(),
        contexts: HashMap::new(),
        target: addr.clone(),
        is_active: true,
        scratch_processes: Arc::new(Mutex::new(Vec::new())),
        scratch_seen: Default::default(),
    };

    let snapshot_addr = ctx.spawn(snapshot_actor);

    // TODO: absurd, must be: loop -> send(SelfTick) -> handler<SelfTick> -> ScanTick
    ctx.spawn_heartbeat(&snapshot_addr, scan_interval_ms, || {
        EventBus::publish(ScanTick);
        NoOp
    });

    //TODO: it broken + need translate
    ui_port.send(UiProcessesPortMsg::SetEmptyStateVisible(true));
    ui_port.send(UiProcessesPortMsg::SetEmptyStateTitle(
        "Waiting For Process Data".into(),
    ));
    ui_port.send(UiProcessesPortMsg::SetEmptyStateMessage(
        "The process list will appear after the agent connects and sends its first snapshot."
            .into(),
    ));

    Ok(())
}
