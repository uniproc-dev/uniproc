use app_contracts::features::settings::{
    SetByteUnits, SetStartPage, SetTheme, SetUpdateInterval, SettingsMsg, SettingsState, ShowSidebarChart,
    UpdateInterval, UpdateIntervalChanged,
};
use guinea::prelude::*;

use super::settings::GeneralSettings;

#[derive(Debug)]
pub struct SettingsActor {
    ui_port: Push<SettingsState>,
    settings: GeneralSettings,
}

impl SettingsActor {
    pub fn new(ui_port: Push<SettingsState>, settings: GeneralSettings) -> Self {
        Self { ui_port, settings }
    }

    fn publish(&self) {
        let state = self.settings.snapshot();
        self.ui_port.send(SettingsMsg::Set {
            theme: state.theme,
            start_page: state.start_page,
            update_interval_ms: state.update_interval_ms,
            byte_units: state.byte_units,
            sidebar_charts: state.sidebar_charts,
        });
    }
}

actor! {
    SettingsActor {
        handlers { SetTheme, SetStartPage, SetUpdateInterval, SetByteUnits, ShowSidebarChart }
    }
}

#[handler]
fn show_sidebar_chart(this: &mut SettingsActor, ShowSidebarChart(chart, shown): ShowSidebarChart) {
    let charts = this.settings.sidebar_charts().with(chart, shown);
    if let Err(err) = this.settings.hidden_sidebar_charts().set(charts.hidden_ids()) {
        tracing::warn!(?err, "sidebar charts setting write failed");
    }
    this.publish();
}

#[handler]
fn set_byte_units(this: &mut SettingsActor, SetByteUnits(units): SetByteUnits) {
    if let Err(err) = this.settings.byte_units().set(units.id().to_string()) {
        tracing::warn!(?err, "byte units setting write failed");
    }
    this.publish();
}

#[handler]
fn set_theme(this: &mut SettingsActor, SetTheme(theme): SetTheme) {
    if let Err(err) = this.settings.theme().set(theme.id().to_string()) {
        tracing::warn!(?err, "theme setting write failed");
    }
    this.publish();
}

#[handler]
fn set_start_page(this: &mut SettingsActor, SetStartPage(page): SetStartPage) {
    if let Err(err) = this.settings.start_page().set(page.id().to_string()) {
        tracing::warn!(?err, "start page setting write failed");
    }
    this.publish();
}

#[handler]
fn set_update_interval(this: &mut SettingsActor, SetUpdateInterval(ms): SetUpdateInterval) {
    let ms = UpdateInterval::clamp(ms).as_millis() as u64;
    if this.settings.update_interval_ms().get() == ms {
        return;
    }
    if let Err(err) = this.settings.update_interval_ms().set(ms) {
        tracing::warn!(?err, "update interval setting write failed");
    }
    GlobalEventBus::publish(UpdateIntervalChanged(ms));
    this.publish();
}
