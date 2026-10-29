use crate::pages::{Services, ServicesSettings};
use crate::routes::ServicesRoute;
use guinea::feature::Install;
use guinea::winui::{Layout, LayoutCx, layout};
use windows_reactor::View;

pub struct ServicesArea;

#[layout]
impl Layout for ServicesArea {
    fn install(_cx: &Install) -> anyhow::Result<Self> {
        _cx.install::<domain::features::services::ServicesFeature>(&())?;
        Ok(Self)
    }

    fn view(&self, cx: &mut LayoutCx<'_, '_, Self>) -> View {
        let nav = cx.navigate::<ServicesRoute>();
        cx.outlet(&nav, |route, cx| match route {
            ServicesRoute::Overview => cx.mount::<Services>(&()),
            ServicesRoute::Settings => cx.mount::<ServicesSettings>(&()),
        })
        .into()
    }
}
