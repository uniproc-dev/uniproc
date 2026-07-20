use crate::features::windows_manager::actor::WindowManagerActor;
use forsl::feature::{AppFeature, AppFeatureInitContext, ContextActorExt};
use forsl::native_windows::slint_factory::SlintWindowRegistry;
use forsl_macros::app_feature;

mod actor;

#[app_feature]
pub fn window_manager_feature(ctx: &mut AppFeatureInitContext) -> anyhow::Result<()> {
    let reg = SlintWindowRegistry::new();

    ctx.shared.insert(reg);
    let reg = ctx.shared.get::<SlintWindowRegistry>().unwrap();

    let actor = WindowManagerActor::new(reg);
    ctx.spawn(actor);

    Ok(())
}
