use crate::ServicesBindings;
use crate::features::services::UiServicesAdapter;
use app_contracts::features::services::{ServiceActionKind, ServiceEntryVm};
use slint::{ComponentHandle, SharedString};

impl UiServicesAdapter {
    fn on_service_action_manual<F>(&self, ui: &crate::AppWindow, handler: F)
    where
        F: Fn(SharedString, ServiceActionKind) + 'static,
    {
        ui.global::<ServicesBindings>()
            .on_service_action(move |name, action| {
                let kind = match action.as_str() {
                    "Start" => ServiceActionKind::Start,
                    "Stop" => ServiceActionKind::Stop,
                    "Restart" => ServiceActionKind::Restart,
                    "Pause" => ServiceActionKind::Pause,
                    "Resume" => ServiceActionKind::Resume,
                    _ => return,
                };
                handler(name, kind);
            });
    }
}

include!(concat!(env!("OUT_DIR"), "/services_bindings_auto.rs"));
