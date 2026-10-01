use guinea::winui::{layout, Layout, LayoutCx};
use windows_reactor::View;

#[derive(Default)]
pub struct ProcessesArea;

#[layout]
impl Layout for ProcessesArea {
    type Params = crate::routes::ProcessesAreaParams;

    fn view(&self, cx: &mut LayoutCx<'_, Self>) -> View {
        cx.outlet()
    }
}
