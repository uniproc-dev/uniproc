use app_contracts::features::settings::{
    SetByteUnits, SetNetworkUnits, SetStartPage, SetTheme, SetUpdateInterval, SettingsMsg, SettingsState, ShowSidebarChart,
    UpdateIntervalChanged,
};
use guinea::prelude::*;

use super::settings::GeneralSettings;

#[derive(Debug)]
pub struct SettingsActor {
    push: Push<SettingsState>,
    settings: GeneralSettings,
}

impl SettingsActor {
    pub fn new(push: Push<SettingsState>, settings: GeneralSettings) -> Self {
        Self { push, settings }
    }

    fn publish(&self) {
        let state = self.settings.snapshot();
        self.push.send(SettingsMsg::Set {
            theme: state.theme,
            start_page: state.start_page,
            update_interval_ms: state.update_interval_ms,
            units: state.units,
            sidebar_charts: state.sidebar_charts,
        });
    }
}

actor! {
    SettingsActor {
        handlers {
            SetTheme,
            SetStartPage,
            SetUpdateInterval,
            SetByteUnits,
            SetNetworkUnits,
            ShowSidebarChart,
        }
        publishes { UpdateIntervalChanged }
    }
}

#[handler]
fn set_theme(In(this, SetTheme(theme), _): In<SettingsActor, SetTheme>) {
    this.settings.theme().set(theme.id().to_string());
    this.publish();
}

#[handler]
fn set_start_page(In(this, SetStartPage(page), _): In<SettingsActor, SetStartPage>) {
    this.settings.start_page().set(page.id().to_string());
    this.publish();
}

#[handler]
fn set_update_interval(In(this, SetUpdateInterval(ms), cx): In<SettingsActor, SetUpdateInterval>) {
    let interval = this.settings.update_interval_ms();
    let before = interval.get();
    interval.set(ms);
    let after = interval.get();
    if after == before {
        return;
    }
    cx.publish(UpdateIntervalChanged(after));
    this.publish();
}

#[handler]
fn set_byte_units(In(this, SetByteUnits(units), _): In<SettingsActor, SetByteUnits>) {
    this.settings.byte_units().set(units.id().to_string());
    this.publish();
}

#[handler]
fn set_network_units(In(this, SetNetworkUnits(units), _): In<SettingsActor, SetNetworkUnits>) {
    this.settings.network_units().set(units.id().to_string());
    this.publish();
}

#[handler]
fn show_sidebar_chart(In(this, ShowSidebarChart(chart, shown), _): In<SettingsActor, ShowSidebarChart>) {
    let charts = this.settings.sidebar_charts().with(chart, shown);
    this.settings.hidden_sidebar_charts().set(charts.hidden_ids());
    this.publish();
}
