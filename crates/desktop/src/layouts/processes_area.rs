use domain::features::processes::{ProcessesDeps, ProcessesFeature};
use guinea::feature::FeatureInitContext;
use guinea::winui::{layout, Layout, LayoutCx};
use windows_reactor::View;

#[derive(Default)]
pub struct ProcessesArea;

#[layout]
impl Layout for ProcessesArea {
    type Params = crate::routes::ProcessesAreaParams;
    type Installs = ProcessesFeature;

    fn install(ctx: &FeatureInitContext, _params: &Self::Params) -> anyhow::Result<Self::Installs> {
        ctx.install(&ctx.require_or_default::<ProcessesDeps>())
    }

    fn view(&self, cx: &mut LayoutCx<'_, Self>) -> View {
        cx.outlet()
    }
}
