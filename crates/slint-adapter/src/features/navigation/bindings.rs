use crate::features::navigation::UiNavigationAdapter;
use slint::ComponentHandle;

impl UiNavigationAdapter {
    fn on_push_manual<F>(&self, ui: &crate::AppWindow, handler: F)
    where
        F: Fn(String) + 'static,
    {
        ui.global::<crate::NavigationBindings>()
            .on_push(move |path| handler(path.to_string()));
    }
}

include!(concat!(env!("OUT_DIR"), "/navigation_bindings_auto.rs"));
