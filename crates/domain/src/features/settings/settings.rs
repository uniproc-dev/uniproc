use amethystate::amethystate;
use app_contracts::features::settings::{AppTheme, ByteUnits, SettingsState, StartPage, UpdateInterval};

#[amethystate(prefix = "general")]
pub struct GeneralSettings {
    #[amestate(default = AppTheme::default().id().to_string())]
    pub theme: String,

    #[amestate(default = StartPage::default().id().to_string())]
    pub start_page: String,

    #[amestate(default = UpdateInterval::Default.as_millis() as u64)]
    pub update_interval_ms: u64,

    #[amestate(default = ByteUnits::default().id().to_string())]
    pub byte_units: String,
}

impl GeneralSettings {
    pub fn theme_choice(&self) -> AppTheme {
        AppTheme::from_id(&self.theme().get()).unwrap_or_default()
    }

    pub fn start_page_choice(&self) -> StartPage {
        StartPage::from_id(&self.start_page().get()).unwrap_or_default()
    }

    pub fn byte_units_choice(&self) -> ByteUnits {
        ByteUnits::from_id(&self.byte_units().get()).unwrap_or_default()
    }

    pub fn update_interval(&self) -> std::time::Duration {
        UpdateInterval::clamp(self.update_interval_ms().get())
    }

    pub fn snapshot(&self) -> SettingsState {
        SettingsState {
            theme: self.theme_choice(),
            start_page: self.start_page_choice(),
            update_interval_ms: self.update_interval().as_millis() as u64,
            byte_units: self.byte_units_choice(),
        }
    }
}
