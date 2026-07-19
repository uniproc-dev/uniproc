use crate::AppWindow;
use app_contracts::features::l10n::L10nPort;
use forsl_macros::port_adapter;
use slint::ComponentHandle;

#[derive(Clone)]
pub struct SlintL10nPort {
    ui: slint::Weak<AppWindow>,
}

impl SlintL10nPort {
    pub fn new(ui: slint::Weak<AppWindow>) -> Self {
        Self { ui }
    }
}

#[port_adapter(backend = "slint", window = AppWindow)]
impl L10nPort for SlintL10nPort {
    fn set_environments(&self, value: String) {
        let Some(ui) = self.ui.upgrade() else { return };
        ui.global::<crate::L10n>().set_environments(value.into());
    }
    fn set_error_connection_lost(&self, value: String) {
        let Some(ui) = self.ui.upgrade() else { return };
        ui.global::<crate::L10n>()
            .set_error_connection_lost(value.into());
    }
    fn set_perfomance_tab(&self, value: String) {
        let Some(ui) = self.ui.upgrade() else { return };
        ui.global::<crate::L10n>().set_perfomance_tab(value.into());
    }
    fn set_search_placeholder(&self, value: String) {
        let Some(ui) = self.ui.upgrade() else { return };
        ui.global::<crate::L10n>()
            .set_search_placeholder(value.into());
    }
    fn set_services_description(&self, value: String) {
        let Some(ui) = self.ui.upgrade() else { return };
        ui.global::<crate::L10n>()
            .set_services_description(value.into());
    }
    fn set_services_display_name(&self, value: String) {
        let Some(ui) = self.ui.upgrade() else { return };
        ui.global::<crate::L10n>()
            .set_services_display_name(value.into());
    }
    fn set_services_group(&self, value: String) {
        let Some(ui) = self.ui.upgrade() else { return };
        ui.global::<crate::L10n>().set_services_group(value.into());
    }
    fn set_services_not_available(&self, value: String) {
        let Some(ui) = self.ui.upgrade() else { return };
        ui.global::<crate::L10n>()
            .set_services_not_available(value.into());
    }
    fn set_services_not_running(&self, value: String) {
        let Some(ui) = self.ui.upgrade() else { return };
        ui.global::<crate::L10n>()
            .set_services_not_running(value.into());
    }
    fn set_services_open_services(&self, value: String) {
        let Some(ui) = self.ui.upgrade() else { return };
        ui.global::<crate::L10n>()
            .set_services_open_services(value.into());
    }
    fn set_services_path_to_executable(&self, value: String) {
        let Some(ui) = self.ui.upgrade() else { return };
        ui.global::<crate::L10n>()
            .set_services_path_to_executable(value.into());
    }
    fn set_services_pid(&self, value: String) {
        let Some(ui) = self.ui.upgrade() else { return };
        ui.global::<crate::L10n>().set_services_pid(value.into());
    }
    fn set_services_properties(&self, value: String) {
        let Some(ui) = self.ui.upgrade() else { return };
        ui.global::<crate::L10n>()
            .set_services_properties(value.into());
    }
    fn set_services_restart(&self, value: String) {
        let Some(ui) = self.ui.upgrade() else { return };
        ui.global::<crate::L10n>()
            .set_services_restart(value.into());
    }
    fn set_services_service_name(&self, value: String) {
        let Some(ui) = self.ui.upgrade() else { return };
        ui.global::<crate::L10n>()
            .set_services_service_name(value.into());
    }
    fn set_services_start(&self, value: String) {
        let Some(ui) = self.ui.upgrade() else { return };
        ui.global::<crate::L10n>().set_services_start(value.into());
    }
    fn set_services_stop(&self, value: String) {
        let Some(ui) = self.ui.upgrade() else { return };
        ui.global::<crate::L10n>().set_services_stop(value.into());
    }
    fn set_settings_save_btn(&self, value: String) {
        let Some(ui) = self.ui.upgrade() else { return };
        ui.global::<crate::L10n>()
            .set_settings_save_btn(value.into());
    }
}
