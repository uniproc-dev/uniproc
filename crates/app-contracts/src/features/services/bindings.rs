use forsl_macros::bindings;
use slint::SharedString;

use super::model::{ServiceActionKind, ServiceEntryVm};

#[bindings]
pub trait UiServicesBindings: 'static {
    #[manual]
    #[slint(arg_types = "string, string")]
    #[tracing(target = "name,kind")]
    fn on_service_action<F>(&self, handler: F)
    where
        F: Fn(SharedString, ServiceActionKind) + 'static;
    fn on_sort_by<F>(&self, handler: F)
    where
        F: Fn(SharedString) + 'static;
    fn on_select_service<F>(&self, handler: F)
    where
        F: Fn(SharedString, i32) + 'static;
    fn on_rows_viewport_changed<F>(&self, handler: F)
    where
        F: Fn(i32, i32) + 'static;
    fn on_column_resized<F>(&self, handler: F)
    where
        F: Fn(SharedString, f32) + 'static;
    #[slint(arg_types = "ServiceEntry")]
    #[slint(import = "import { ServiceEntry } from \"./globals.slint\";")]
    fn on_open_properties_window<F>(&self, handler: F)
    where
        F: Fn(ServiceEntryVm) + 'static;
}
