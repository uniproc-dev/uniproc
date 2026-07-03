use app_core::actor::registry::ActorRegistry;
use framework::feature::{AppFeature, AppFeatureInitContext};
use macros::app_feature;

//TODO: move to framework layer
#[app_feature]
pub fn test_discovery_feature(ctx: &mut AppFeatureInitContext) -> anyhow::Result<()> {
    ctx.shared.insert(ActorRegistry::default());
    Ok(())
}
