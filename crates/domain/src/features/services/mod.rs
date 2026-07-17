use crate::features::services::application::actor::ServiceActor;
use crate::features::services::application::snapshot_actor::ServiceSnapshotActor;
use crate::features::services::settings::ServiceSettings;
use crate::features::services::view::ServiceTable;
use app_contracts::capabilities;
use app_contracts::features::agents::ScanTick;
use app_contracts::features::services::{
    ServicesBinder, ServicesWindowRegister, UiServicesBindings, UiServicesPort,
};
use context::page_status::RouteStatusRegistry;
use forsl::app::Window;
use forsl::feature::{
    ContextActorExt, ContextReactorExt, ContextStoreExt, FeatureContextState, WindowFeature,
    WindowFeatureInitContext,
};
use forsl::native_windows::slint_factory::SlintWindowRegistry;
use macros::window_feature;
use std::borrow::Cow;
use std::collections::HashSet;

pub mod application;

mod scanner;
mod settings;
mod view;

#[window_feature]
pub fn services_feature<TWindow, P>(
    ctx: &mut WindowFeatureInitContext<TWindow>,
    ui_port: P,
) -> anyhow::Result<()>
where
    TWindow: Window,
    P: UiServicesPort + UiServicesBindings + ServicesWindowRegister + Clone + 'static,
{
    let store = ctx.store();
    let settings = ServiceSettings::new(&store)?;

    let reg = ctx.shared.get::<SlintWindowRegistry>().unwrap();

    let service_actor = ServiceActor {
        registry: reg.clone(),
        table: ServiceTable::new(settings.clone())?,
        ui_port: ui_port.clone(),
        route_status: ctx.shared.get::<RouteStatusRegistry>().unwrap(),
        is_active: true,
        active_context_key: Cow::Borrowed("host"),
        pending: HashSet::new(),
        ctx_state: FeatureContextState::new(ctx.window_id, capabilities::SERVICES),
    };

    let addr = ctx.actor_builder(service_actor).ui_bind(&ui_port);

    let snapshot_actor = ServiceSnapshotActor {
        target: addr.clone(),
        is_active: true,
    };

    let snapshot_addr = ctx.spawn(snapshot_actor);

    ctx.spawn_heartbeat(&snapshot_addr, settings.scan_interval_ms(), || ScanTick);

    ui_port.register(&reg);

    Ok(())
}
