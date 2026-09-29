use guinea::prelude::*;

use super::messages::SettingsMsg;
use super::model::{AppTheme, ByteUnits, SidebarCharts, StartPage, UpdateInterval};

#[derive(Clone, PartialEq, Debug)]
pub struct SettingsState {
    pub theme: AppTheme,
    pub start_page: StartPage,
    pub update_interval_ms: u64,
    pub byte_units: ByteUnits,
    pub sidebar_charts: SidebarCharts,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self {
            theme: AppTheme::default(),
            start_page: StartPage::default(),
            update_interval_ms: UpdateInterval::Default.as_millis() as u64,
            byte_units: ByteUnits::default(),
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
        byte_units,
        sidebar_charts,
    }: SettingsMsg,
) {
    this.theme = theme;
    this.start_page = start_page;
    this.update_interval_ms = update_interval_ms;
    this.byte_units = byte_units;
    this.sidebar_charts = sidebar_charts;
}
