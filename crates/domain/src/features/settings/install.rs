use app_contracts::features::settings::SettingsState;
use guinea::prelude::*;
use guinea_plugin_store::StoreAccess;
use amethystate::Store;

use super::actor::SettingsActor;
use super::settings::GeneralSettings;

feature! {
    pub SettingsFeature {
        exports { SettingsState }
    }
}

#[installs]
fn settings(cx: &Install, store: Store) -> anyhow::Result<SettingsFeature> {
    let settings = store.try_settings::<GeneralSettings>()?;
    let seed = settings.snapshot();

    let (state, _) = cx
        .state::<SettingsState>()
        .seed(seed)
        .driven_by(|push| SettingsActor::new(push, settings));

    Ok(SettingsFeature(state))
}
