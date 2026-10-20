use domain::features::system::{SystemDeps, SystemFeature};
use guinea::feature::Install;
use guinea::winui::{layout, Layout, LayoutCx};
use windows_reactor::View;

use crate::routes::Route;
use crate::pages::{System, SystemTools};

#[derive(Default)]
pub struct SystemArea;

#[layout]
impl Layout for SystemArea {
    fn install(cx: &Install, deps: Option<SystemDeps>) -> anyhow::Result<Self> {
        cx.install::<SystemFeature>(&deps.unwrap_or_default())?;
        Ok(Self)
    }

    fn view(&self, cx: &mut LayoutCx<'_, '_, Self>) -> View {
        let navigate = cx.navigate::<Route>();
        cx.outlet(&navigate, |route, cx| match route {
            Route::System {} => cx.mount::<System>(&()),
            Route::SystemTools {} => cx.mount::<SystemTools>(&()),
            _ => unreachable!("route outside SystemArea"),
        }).into()
    }
}
