use crate::features::window_actions::UiWindowActionsAdapter;
use app_contracts::features::window_actions::{ResizeEdge, UiWindowActionsPort, UiWindowActionsPortMsg};
use i_slint_backend_winit::WinitWindowAccessor;
use forsl_macros::port_adapter;
use slint::ComponentHandle;
use winit::window::ResizeDirection;

#[port_adapter(backend = "slint", window = AppWindow)]
impl UiWindowActionsPort for UiWindowActionsAdapter {
    fn send(&self, ui: &AppWindow, msg: UiWindowActionsPortMsg) {
        match msg {
            UiWindowActionsPortMsg::Drag => {
                ui.window().with_winit_window(|w| {
                    let _ = w.drag_window();
                });
            }
            UiWindowActionsPortMsg::Close => {
                let _ = ui.hide();
            }
            UiWindowActionsPortMsg::Minimize => {
                ui.window().set_minimized(true);
            }
            UiWindowActionsPortMsg::ToggleMaximize => {
                let window = ui.window();
                window.set_maximized(!window.is_maximized());
            }
            UiWindowActionsPortMsg::Resize(edge) => {
                let direction = match edge {
                    ResizeEdge::North => ResizeDirection::North,
                    ResizeEdge::South => ResizeDirection::South,
                    ResizeEdge::West => ResizeDirection::West,
                    ResizeEdge::East => ResizeDirection::East,
                    ResizeEdge::NorthWest => ResizeDirection::NorthWest,
                    ResizeEdge::NorthEast => ResizeDirection::NorthEast,
                    ResizeEdge::SouthWest => ResizeDirection::SouthWest,
                    ResizeEdge::SouthEast => ResizeDirection::SouthEast,
                };
                ui.window().with_winit_window(|w| {
                    let _ = w.drag_resize_window(direction);
                });
            }
        }
    }
}
