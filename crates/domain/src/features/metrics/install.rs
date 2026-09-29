use app_contracts::features::agents::{WindowsMachineSample, WindowsReportMessage};
use app_contracts::features::metrics::MetricsState;
use guinea::prelude::*;

use super::actor::MetricsActor;

feature! {
    pub MetricsFeature {
        exports { MetricsState }
    }
}

#[installs]
fn metrics(cx: &FeatureInitContext) -> anyhow::Result<MetricsFeature> {
    let (metrics, addr) = cx.state::<MetricsState>().driven_by(MetricsActor::new);
    addr.subscribe_on::<WindowsReportMessage>(Bus::Global);
    addr.subscribe_on::<WindowsMachineSample>(Bus::Global);
    Ok(MetricsFeature(metrics))
}
