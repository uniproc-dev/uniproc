use amethystate::amethystate;
use amethystate::store::{CheckContext, Invalid};
use app_contracts::features::settings::{
    AppTheme, ByteUnits, NetworkUnits, SettingsState, SidebarCharts, StartPage, Units, UpdateInterval,
};

#[amethystate(prefix = "general")]
pub struct GeneralSettings {
    #[amestate(default = AppTheme::default().id().to_string())]
    pub theme: String,

    #[amestate(default = StartPage::default().id().to_string())]
    pub start_page: String,

    #[amestate(default = UpdateInterval::Default.as_millis() as u64, check = fit_update_interval)]
    pub update_interval_ms: u64,

    #[amestate(default = ByteUnits::default().id().to_string())]
    pub byte_units: String,

    #[amestate(default = NetworkUnits::default().id().to_string())]
    pub network_units: String,

    #[amestate(default = SidebarCharts::default().hidden_ids())]
    pub hidden_sidebar_charts: String,
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

    pub fn network_units_choice(&self) -> NetworkUnits {
        NetworkUnits::from_id(&self.network_units().get()).unwrap_or_default()
    }

    pub fn units(&self) -> Units {
        Units {
            bytes: self.byte_units_choice(),
            network: self.network_units_choice(),
        }
    }

    pub fn sidebar_charts(&self) -> SidebarCharts {
        SidebarCharts::from_hidden_ids(&self.hidden_sidebar_charts().get())
    }

    pub fn update_interval(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.update_interval_ms().get())
    }

    pub fn snapshot(&self) -> SettingsState {
        SettingsState {
            theme: self.theme_choice(),
            start_page: self.start_page_choice(),
            update_interval_ms: self.update_interval().as_millis() as u64,
            units: self.units(),
            sidebar_charts: self.sidebar_charts(),
        }
    }
}

fn fit_update_interval(ms: &mut u64, _: &CheckContext) -> Result<(), Invalid> {
    *ms = UpdateInterval::clamp(*ms).as_millis() as u64;
    Ok(())
}
