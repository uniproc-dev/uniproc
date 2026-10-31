use domain::features::system::{SystemDeps, SystemFeature};
use guinea::feature::Install;
use guinea::winui::{mount, layout, Layout, LayoutCx};
use windows_reactor::View;

use crate::routes::SystemRoute;
use crate::pages::{System, SystemTools};

#[derive(Default)]
pub struct SystemArea;

#[layout]
impl Layout for SystemArea {
    fn install(cx: &Install, deps: SystemDeps) -> anyhow::Result<Self> {
        cx.install::<SystemFeature>(&deps)?;
        Ok(Self)
    }

    fn view(&self, cx: &mut LayoutCx<'_, '_, Self>) -> View {
        cx.outlet::<SystemRoute>(|route| match route {
            SystemRoute::Overview => mount::<System>(()),
            SystemRoute::Tools => mount::<SystemTools>(()),
        }).into()
    }
}
