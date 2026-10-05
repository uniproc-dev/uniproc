use app_contracts::features::settings::SettingsState;
use guinea::prelude::*;
use guinea_plugin_store::StoreAccess;

use super::actor::SettingsActor;
use super::settings::GeneralSettings;

feature! {
    pub SettingsFeature {
        exports { SettingsState }
    }
}

#[installs]
fn settings(cx: &FeatureInitContext) -> anyhow::Result<SettingsFeature> {
    let settings = cx.settings::<GeneralSettings>();
    let seed = settings.snapshot();

    let (state, _) = cx
        .state::<SettingsState>()
        .seed(seed)
        .driven_by(|push| SettingsActor::new(push, settings));

    Ok(SettingsFeature(state))
}
