use guinea::prelude::Event;
use serde::Deserialize;

use super::model::{AppTheme, StartPage};

#[derive(Clone, Copy)]
pub enum SettingsMsg {
    Set {
        theme: AppTheme,
        start_page: StartPage,
        update_interval_ms: u64,
    },
}

#[derive(Clone, Debug, Deserialize, guinea::Remote)]
#[remote(action)]
pub struct SetTheme(pub AppTheme);

#[derive(Clone, Debug, Deserialize, guinea::Remote)]
#[remote(action)]
pub struct SetStartPage(pub StartPage);

#[derive(Clone, Debug, Deserialize, guinea::Remote)]
#[remote(action)]
pub struct SetUpdateInterval(pub u64);

#[derive(Clone, Debug, Event, Deserialize, guinea::Remote)]
#[remote(event)]
pub struct UpdateIntervalChanged(pub u64);
