use domain::features::processes::{ProcessesDeps, ProcessesFeature};
use guinea::feature::Install;
use guinea::winui::{layout, Layout, LayoutCx};
use windows_reactor::View;

use crate::routes::Route;
use crate::pages::{Processes, ProcessesSettings};

#[derive(Default)]
pub struct ProcessesArea;

#[layout]
impl Layout for ProcessesArea {
    fn install(cx: &Install, deps: ProcessesDeps) -> anyhow::Result<Self> {
        cx.install::<ProcessesFeature>(&deps)?;
        Ok(Self)
    }

    fn view(&self, cx: &mut LayoutCx<'_, '_, Self>) -> View {
        let navigate = cx.navigate::<Route>();
        cx.outlet(&navigate, |route, cx| match route {
            Route::Processes {} => cx.mount::<Processes>(&()),
            Route::ProcessesSettings {} => cx.mount::<ProcessesSettings>(&()),
            _ => unreachable!("route outside ProcessesArea"),
        }).into()
    }
}
