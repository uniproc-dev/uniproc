use domain::features::system::{SystemDeps, SystemFeature};
use guinea::feature::Install;
use guinea::winui::{layout, Layout, LayoutCx};
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
        let navigate = cx.navigate::<SystemRoute>();
        cx.outlet(&navigate, |route, cx| match route {
            SystemRoute::Overview => cx.mount::<System>(&()),
            SystemRoute::Tools => cx.mount::<SystemTools>(&()),
        }).into()
    }
}
