use app_contracts::features::services::ServicesState;
use guinea::prelude::*;

use super::actor::ServicesActor;
use crate::features::processes::ProcessesDeps;
use crate::features::system::SystemDeps;

feature! {
    pub ServicesFeature {
        exports { ServicesState }
    }
}

#[installs]
fn services(cx: &Install, processes: ProcessesDeps, system: SystemDeps) -> anyhow::Result<ServicesFeature> {
    let (services, _) = cx.state::<ServicesState>().driven_by(move |push| ServicesActor::new(push, processes.shell, system.launch));

    Ok(ServicesFeature(services))
}
