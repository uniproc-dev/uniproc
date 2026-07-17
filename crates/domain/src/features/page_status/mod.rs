use context::page_status::RouteStatusRegistry;
use forsl::feature::{AppFeature, AppFeatureInitContext};
use macros::app_feature;
use std::sync::Arc;

#[app_feature]
pub fn page_status_feature(ctx: &mut AppFeatureInitContext) -> anyhow::Result<()> {
    let registry = Arc::new(RouteStatusRegistry::new());
    ctx.shared.insert_arc(registry);
    Ok(())
}
