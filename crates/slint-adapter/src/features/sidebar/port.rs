use crate::features::sidebar::UiSidebarAdapter;
use app_contracts::features::sidebar::{UiSidebarPort, UiSidebarPortMsg};
use forsl_macros::port_adapter;
use slint::ComponentHandle;
use slint::private_unstable_api::re_exports::Coord;

#[port_adapter(backend = "slint", window = AppWindow)]
impl UiSidebarPort for UiSidebarAdapter {
    fn send(&self, ui: &AppWindow, msg: UiSidebarPortMsg) {
        let sidebar = ui.global::<crate::Sidebar>();
        match msg {
            UiSidebarPortMsg::SetSwitchTransition { from_index, to_index, progress } => {
                sidebar.set_switch_from_index(from_index);
                sidebar.set_switch_to_index(to_index);
                sidebar.set_switch_progress(progress);
            }
            UiSidebarPortMsg::SetSideBarWidth(width) => {
                sidebar.set_side_bar_width(width as Coord)
            }
            UiSidebarPortMsg::SetSwitchProgress(progress) => sidebar.set_switch_progress(progress),
            UiSidebarPortMsg::SetContentVisible(visible) => {
                sidebar.set_content_visible(visible)
            }
        }
    }
}
