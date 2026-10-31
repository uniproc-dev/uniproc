use domain::features::processes::{ProcessesDeps, ProcessesFeature};
use guinea::feature::Install;
use guinea::winui::{mount, layout, Layout, LayoutCx};
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
        cx.outlet::<ProcessesRoute>(|route| match route {
            ProcessesRoute::Overview => mount::<Processes>(()),
            ProcessesRoute::Settings => mount::<ProcessesSettings>(()),
        }).into()
    }
}
