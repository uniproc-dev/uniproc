use app_contracts::features::agents::WindowsReportMessage;
use app_contracts::features::services::ServicesState;
use guinea::prelude::*;

use super::actor::ServicesActor;

feature! {
    pub ServicesFeature {
        exports { ServicesState }
    }
}

#[installs]
fn services(cx: &FeatureInitContext) -> anyhow::Result<ServicesFeature> {
    let (services, addr) = cx.state::<ServicesState>().driven_by(ServicesActor::new);
    addr.subscribe_on::<WindowsReportMessage>(Bus::Global);

    Ok(ServicesFeature(services))
}
