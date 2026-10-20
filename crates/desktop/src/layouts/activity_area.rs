use guinea::winui::{layout, Layout, LayoutCx};
use windows_reactor::View;

use crate::routes::Route;
use crate::pages::{Activity, ActivityGroups};

#[derive(Default)]
pub struct ActivityArea;

#[layout]
impl Layout for ActivityArea {
    fn view(&self, cx: &mut LayoutCx<'_, '_, Self>) -> View {
        let navigate = cx.navigate::<Route>();
        cx.outlet(&navigate, |route, cx| match route {
            Route::Activity {} => cx.mount::<Activity>(&()),
            Route::ActivityGroups {} => cx.mount::<ActivityGroups>(&()),
            _ => unreachable!("route outside ActivityArea"),
        }).into()
    }
}
