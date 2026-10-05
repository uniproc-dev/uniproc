use app_contracts::l10n::L10n;
use domain::features::activity::ActivityFeature;
use domain::features::agents::AgentsFeature;
use guinea::prelude::*;
use guinea_plugin_l10n::L10nPlugin;
use guinea_plugin_single_instance::SingleInstancePlugin;
use guinea_plugin_store::amethystate::store::OnUnreadable;
use guinea_plugin_store::StorePlugin;

use crate::meta;

app! {
    pub App {
        installs { L10nPlugin<L10n>, ActivityFeature }
    }
}

#[installs]
fn app(app: &mut FeatureBuilder) -> anyhow::Result<App> {
    app.plugin(SingleInstancePlugin::new())?;
    app.plugin(settings_store(StorePlugin::for_app(meta::APP_NAME, "settings")))?;
    let l10n = app.plugin(L10nPlugin::<L10n>::new("en"))?;

    #[cfg(debug_assertions)]
    if std::env::var_os("UNIPROC_NO_DEVTOOLS").is_none() {
        app.plugin(guinea_plugin_devtools::DevToolsPlugin::new().launch(true))?;
    }

    app.feature(AgentsFeature)?;
    Ok(App(l10n, app.feature(ActivityFeature)?))
}

#[cfg(test)]
pub fn with_fakes(app: &mut FeatureBuilder) -> anyhow::Result<App> {
    app.plugin(StorePlugin::in_memory())?;
    let l10n = app.plugin(L10nPlugin::<L10n>::new("en"))?;
    app.feature(crate::test_agent::FakeAgentFeature)?;
    Ok(App(l10n, app.feature(ActivityFeature)?))
}

fn settings_store(plugin: StorePlugin) -> StorePlugin {
    let plugin = plugin
        .or_in_memory()
        .configure(|store| store.rules(|rules| rules.on_unreadable(OnUnreadable::UseDefault)));
    #[cfg(debug_assertions)]
    let plugin = plugin.configure(|store| store.rules(|rules| rules.on_undeclared(guinea_plugin_store::amethystate::store::OnUndeclared::Drop)));
    plugin
}

#[cfg(test)]
mod tests {
    use domain::features::agents::settings::AgentSettings;
    use guinea::app::Harness;
    use guinea_plugin_store::amethystate::store::builder::StoreBuilder;
    use guinea_plugin_store::{Persistence, StoreAccess, StorePlugin};

    use super::settings_store;

    #[guinea::test(iterations = 1)]
    fn a_key_no_setting_declares_any_more_is_dropped_in_a_debug_build(h: &mut Harness) {
        let dir = tempfile::tempdir().unwrap();
        let at = dir.path().join("settings");
        let (earlier, _) = StoreBuilder::new(&at).migrate().unwrap();
        earlier.kv().namespace("agents").set("scan_interval_ms", &500u64).unwrap();
        AgentSettings::new_with(&earlier).ping_interval_ms().set(1000);
        earlier.close().unwrap();
        drop(earlier);

        h.plugin(settings_store(StorePlugin::at(&at))).unwrap();

        let kept: Vec<String> = h
            .segment()
            .store()
            .unwrap()
            .scan_keys(["agents"])
            .unwrap()
            .iter()
            .map(ToString::to_string)
            .collect();
        assert!(!kept.iter().any(|key| key.ends_with("scan_interval_ms")), "{kept:?}");
        assert!(kept.iter().any(|key| key.ends_with("ping_interval_ms")), "{kept:?}");
    }

    #[guinea::test(iterations = 1)]
    fn a_setting_that_will_not_read_back_opens_on_its_default(h: &mut Harness) {
        let dir = tempfile::tempdir().unwrap();
        let at = dir.path().join("settings");
        let (earlier, _) = StoreBuilder::new(&at).migrate().unwrap();
        earlier.set(["agents", "ping_interval_ms"], &"often").unwrap();
        earlier.close().unwrap();
        drop(earlier);

        h.plugin(settings_store(StorePlugin::at(&at))).unwrap();

        let opened = h.segment().try_settings::<AgentSettings>().map(|settings| settings.ping_interval_ms().get());
        assert!(matches!(opened, Ok(2000)), "{opened:?}");
    }

    #[guinea::test(iterations = 1)]
    fn a_store_file_that_will_not_open_leaves_the_settings_in_memory(h: &mut Harness) {
        let dir = tempfile::tempdir().unwrap();
        let at = dir.path().join("settings");
        std::fs::write(at.with_extension("redb"), "{ this never finished").unwrap();

        let installed = h.plugin(settings_store(StorePlugin::at(&at))).map(|_| ());

        assert!(installed.is_ok(), "{installed:?}");
        let in_memory = h.segment().try_require::<Persistence>().map(|persistence| persistence.is_in_memory());
        assert_eq!(in_memory, Some(true));
        assert_eq!(h.segment().settings::<AgentSettings>().ping_interval_ms().get(), 2000);
    }
}
