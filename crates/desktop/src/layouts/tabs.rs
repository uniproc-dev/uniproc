use guinea::winui::{layout, Layout, LayoutCx};
use windows_reactor::View;

#[derive(Default)]
pub struct TabsLayout;

#[layout]
impl Layout for TabsLayout {
    type Params = crate::routes::TabsLayoutParams;

    fn view(&self, cx: &mut LayoutCx<'_, Self>) -> View {
        cx.outlet()
    }
}
