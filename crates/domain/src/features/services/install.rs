use app_contracts::features::services::ServicesState;
use guinea::prelude::*;

use super::actor::ServicesActor;

feature! {
    pub ServicesFeature {
        exports { ServicesState }
    }
}

#[installs]
fn services(cx: &Install) -> anyhow::Result<ServicesFeature> {
    let (services, _) = cx.state::<ServicesState>().driven_by(ServicesActor::new);

    Ok(ServicesFeature(services))
}
