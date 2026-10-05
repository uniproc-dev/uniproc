use guinea::feature::FeatureInitContext;
use guinea::winui::{layout, Layout, LayoutCx};
use windows_reactor::View;

#[derive(Default)]
pub struct ActivityArea;

#[layout]
impl Layout for ActivityArea {
    type Params = crate::routes::ActivityAreaParams;
    type Installs = ();

    fn install(_ctx: &FeatureInitContext, _params: &Self::Params) -> anyhow::Result<Self::Installs> {
        Ok(())
    }

    fn view(&self, cx: &mut LayoutCx<'_, '_, Self>) -> View {
        cx.outlet()
    }
}
