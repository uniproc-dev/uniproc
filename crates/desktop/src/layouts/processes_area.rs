use domain::features::processes::{ProcessesDeps, ProcessesFeature};
use guinea::feature::Install;
use guinea::winui::{layout, Layout, LayoutCx};
use windows_reactor::View;

use crate::routes::ProcessesRoute;
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
        let navigate = cx.navigate::<ProcessesRoute>();
        cx.outlet(&navigate, |route, cx| match route {
            ProcessesRoute::Overview => cx.mount::<Processes>(&()),
            ProcessesRoute::Settings => cx.mount::<ProcessesSettings>(&()),
        }).into()
    }
}
