use crate::features::window_actions::UiWindowActionsAdapter;
use crate::{WindowAdapter, WindowSize};
use app_contracts::features::window_actions::{ResizeEdge, WindowBreakpoint};
use slint::ComponentHandle;

impl UiWindowActionsAdapter {
    fn on_start_resize_manual<F>(&self, ui: &crate::AppWindow, handler: F)
    where
        F: Fn(ResizeEdge) + 'static,
    {
        ui.on_start_resize(move |v| {
            let edge = match v {
                0 => ResizeEdge::North,
                1 => ResizeEdge::South,
                2 => ResizeEdge::West,
                3 => ResizeEdge::East,
                4 => ResizeEdge::NorthWest,
                5 => ResizeEdge::NorthEast,
                6 => ResizeEdge::SouthWest,
                7 => ResizeEdge::SouthEast,
                _ => return,
            };
            handler(edge);
        });
    }

    fn on_config_changed_manual<F>(&self, ui: &crate::AppWindow, handler: F)
    where
        F: Fn(WindowBreakpoint, u64) + 'static,
    {
        let ui_weak = self.ui.clone();
        ui.global::<WindowAdapter>().on_size_changed(move |size| {
            let Some(ui) = ui_weak.upgrade() else { return };
            let adapter = ui.global::<WindowAdapter>();
            let breakpoint = match size {
                WindowSize::Sm => WindowBreakpoint::Sm,
                WindowSize::Md => WindowBreakpoint::Md,
                WindowSize::Lg => WindowBreakpoint::Lg,
            };
            handler(breakpoint, adapter.get_window_width() as u64);
        });
    }
}

include!(concat!(env!("OUT_DIR"), "/window_actions_bindings_auto.rs"));
