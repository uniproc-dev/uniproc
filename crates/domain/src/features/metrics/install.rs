use app_contracts::features::metrics::MetricsState;
use guinea::prelude::*;

use super::actor::MetricsActor;

feature! {
    pub MetricsFeature {
        exports { MetricsState }
    }
}

#[installs]
fn metrics(cx: &Install) -> anyhow::Result<MetricsFeature> {
    let (metrics, _) = cx.state::<MetricsState>().driven_by(MetricsActor::new);
    Ok(MetricsFeature(metrics))
}
