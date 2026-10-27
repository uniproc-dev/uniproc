use guinea::winui::{layout, Layout, LayoutCx};
use windows_reactor::View;

use crate::routes::ActivityRoute;
use crate::pages::{Activity, ActivityGroups};

#[derive(Default)]
pub struct ActivityArea;

#[layout]
impl Layout for ActivityArea {
    fn view(&self, cx: &mut LayoutCx<'_, '_, Self>) -> View {
        let navigate = cx.navigate::<ActivityRoute>();
        cx.outlet(&navigate, |route, cx| match route {
            ActivityRoute::Overview => cx.mount::<Activity>(&()),
            ActivityRoute::Groups => cx.mount::<ActivityGroups>(&()),
        }).into()
    }
}
