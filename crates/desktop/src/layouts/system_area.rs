use domain::features::system::{SystemDeps, SystemFeature};
use guinea::feature::FeatureInitContext;
use guinea::winui::{layout, Layout, LayoutCx};
use windows_reactor::View;

#[derive(Default)]
pub struct SystemArea;

#[layout]
impl Layout for SystemArea {
    type Params = crate::routes::SystemAreaParams;
    type Installs = SystemFeature;

    fn install(ctx: &FeatureInitContext, _params: &Self::Params) -> anyhow::Result<Self::Installs> {
        ctx.install(&ctx.require_or_default::<SystemDeps>())
    }

    fn view(&self, cx: &mut LayoutCx<'_, Self>) -> View {
        cx.outlet()
    }
}
