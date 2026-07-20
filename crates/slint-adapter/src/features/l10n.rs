use crate::AppWindow;
use app_contracts::features::l10n::L10nStrings;
use forsl_core::l10n::L10nSink;
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

impl L10nSink<L10nStrings> for SlintL10nPort {
    fn load(&self, strings: L10nStrings) {
        let Some(ui) = self.ui.upgrade() else { return };
        let l10n = ui.global::<crate::L10n>();
        include!(concat!(env!("OUT_DIR"), "/l10n_load_body.rs"));
    }
}
