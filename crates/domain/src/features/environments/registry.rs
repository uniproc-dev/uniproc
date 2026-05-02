use app_contracts::features::environments::{
    DiscoveryReport, EnvironmentDescriptor, EnvironmentRegistryChanged,
};
use app_core::actor::{Context, ManagedActor};
use macros::{actor_manifest, handler};
use std::borrow::Cow;
use std::collections::HashMap;

#[actor_manifest]
impl ManagedActor for EnvironmentRegistryActor {
    type Bus = bus!(DiscoveryReport);
    type Handlers = handlers!(@DiscoveryReport);
    type Signals = bus!(EnvironmentRegistryChanged);
}

pub struct EnvironmentRegistryActor {
    reports: HashMap<Cow<'static, str>, Vec<EnvironmentDescriptor>>,
}

impl EnvironmentRegistryActor {
    pub fn new() -> Self {
        Self {
            reports: HashMap::new(),
        }
    }

    fn broadcast_change(&self, ctx: &Context<EnvironmentRegistryActor>) {
        let all_envs = self.reports.values().flatten().cloned().collect::<Vec<_>>();

        ctx.publish(EnvironmentRegistryChanged {
            environments: all_envs,
        });
    }
}

#[handler]
fn handle_report(
    this: &mut EnvironmentRegistryActor,
    msg: DiscoveryReport,
    ctx: &Context<EnvironmentRegistryActor>,
) {
    this.reports.insert(msg.provider_id, msg.items);
    this.broadcast_change(ctx);
}
