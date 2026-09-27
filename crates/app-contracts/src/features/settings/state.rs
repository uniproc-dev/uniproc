use guinea::prelude::*;

use super::messages::SettingsMsg;
use super::model::{AppTheme, StartPage, UpdateInterval};

#[derive(Clone, PartialEq, Debug)]
pub struct SettingsState {
    pub theme: AppTheme,
    pub start_page: StartPage,
    pub update_interval_ms: u64,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self {
            theme: AppTheme::default(),
            start_page: StartPage::default(),
            update_interval_ms: UpdateInterval::Default.as_millis() as u64,
        }
    }
}

impl Reducer for SettingsState {
    type Update = SettingsMsg;

    fn reduce(&mut self, update: SettingsMsg) {
        match update {
            SettingsMsg::Set {
                theme,
                start_page,
                update_interval_ms,
            } => {
                self.theme = theme;
                self.start_page = start_page;
                self.update_interval_ms = update_interval_ms;
            }
        }
    }
}
