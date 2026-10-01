use guinea::prelude::*;

use super::messages::SettingsMsg;
use super::model::{AppTheme, SidebarCharts, StartPage, Units, UpdateInterval};

#[derive(Clone, PartialEq, Debug)]
pub struct SettingsState {
    pub theme: AppTheme,
    pub start_page: StartPage,
    pub update_interval_ms: u64,
    pub units: Units,
    pub sidebar_charts: SidebarCharts,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self {
            theme: AppTheme::default(),
            start_page: StartPage::default(),
            update_interval_ms: UpdateInterval::Default.as_millis() as u64,
            units: Units::default(),
            sidebar_charts: SidebarCharts::default(),
        }
    }
}

#[reducer]
fn settings(
    this: &mut SettingsState,
    SettingsMsg::Set {
        theme,
        start_page,
        update_interval_ms,
        units,
        sidebar_charts,
    }: SettingsMsg,
) {
    this.theme = theme;
    this.start_page = start_page;
    this.update_interval_ms = update_interval_ms;
    this.units = units;
    this.sidebar_charts = sidebar_charts;
}
