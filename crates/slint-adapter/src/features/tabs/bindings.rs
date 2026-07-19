use crate::features::tabs::UiTabsAdapter;
use slint::ComponentHandle;

impl UiTabsAdapter {
    fn on_request_tab_switch_manual<F>(&self, ui: &crate::AppWindow, handler: F)
    where
        F: Fn(String) + 'static,
    {
        ui.global::<crate::TabsBindings>()
            .on_request_tab_switch(move |context_key| handler(context_key.to_string()));
    }

    fn on_request_tab_close_manual<F>(&self, ui: &crate::AppWindow, handler: F)
    where
        F: Fn(String) + 'static,
    {
        ui.global::<crate::TabsBindings>()
            .on_request_tab_close(move |context_key| handler(context_key.to_string()));
    }

    fn on_request_tab_add_manual<F>(&self, ui: &crate::AppWindow, handler: F)
    where
        F: Fn(String) + 'static,
    {
        ui.global::<crate::TabsBindings>()
            .on_request_tab_add(move |context_key| handler(context_key.to_string()));
    }
}

include!(concat!(env!("OUT_DIR"), "/tabs_bindings_auto.rs"));
