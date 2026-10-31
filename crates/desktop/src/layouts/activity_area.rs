use guinea::winui::{mount, layout, Layout, LayoutCx};
use windows_reactor::View;

use crate::routes::ActivityRoute;
use crate::pages::{Activity, ActivityGroups};

#[derive(Default)]
pub struct ActivityArea;

#[layout]
impl Layout for ActivityArea {
    fn view(&self, cx: &mut LayoutCx<'_, '_, Self>) -> View {
        cx.outlet::<ActivityRoute>(|route| match route {
            ActivityRoute::Overview => mount::<Activity>(()),
            ActivityRoute::Groups => mount::<ActivityGroups>(()),
        }).into()
    }
}
