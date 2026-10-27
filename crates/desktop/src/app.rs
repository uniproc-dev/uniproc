use amethystate::Store;
use app_contracts::l10n::L10n;
use domain::features::activity::ActivityFeature;
use domain::features::agents::AgentsFeature;
use guinea::prelude::*;
use guinea::winui::Window;
use guinea_plugin_l10n::{L10nConfig, L10nPlugin};
use guinea_plugin_single_instance::{SingleInstanceConfig, SingleInstancePlugin};
use guinea_plugin_store::amethystate::store::OnUnreadable;
use guinea_plugin_store::{StoreConfig, StorePlugin};

use crate::layouts::MainWindow;
use crate::meta;
use crate::platform::{Platform, PlatformDeps};

pub fn app(app: &mut FeatureBuilder) -> anyhow::Result<Window> {
    app.plugin::<Platform>(PlatformDeps::default())?;
    app.plugin::<SingleInstancePlugin>(SingleInstanceConfig::new(&guinea::app_meta!()))?;
    let StorePlugin(store, _) = install_store(app, settings_store(StoreConfig::for_app(meta::APP_NAME, "settings")))?;
    app.plugin::<L10nPlugin<L10n>>(L10nConfig::new("en").remembered_in(store.clone()))?;

    #[cfg(debug_assertions)]
    if std::env::var_os("UNIPROC_NO_DEVTOOLS").is_none() {
        app.plugin::<guinea_plugin_devtools::DevToolsPlugin>(
            guinea_plugin_devtools::DevToolsConfig::new().for_app(guinea::app_meta!()).launch(true),
        )?;
    }

    app.install::<AgentsFeature>()?;
    app.install::<ActivityFeature>()?;
    window(&store)
}

fn install_store(app: &mut FeatureBuilder, config: StoreConfig) -> anyhow::Result<StorePlugin> {
    app.plugin::<StorePlugin>(config)
}

fn window(store: &Store) -> anyhow::Result<Window> {
    Ok(Window::of::<MainWindow>()
        .title(meta::WINDOW_TITLE)
        .at(crate::route_memory::restore(store)?))
}

#[cfg(test)]
pub fn with_fakes(app: &mut FeatureBuilder, deps: PlatformDeps) -> anyhow::Result<Store> {
    app.plugin::<Platform>(deps)?;
    let StorePlugin(store, _) = install_store(app, StoreConfig::in_memory())?;
    app.plugin::<L10nPlugin<L10n>>(L10nConfig::new("en").remembered_in(store.clone()))?;
    app.install::<crate::test_agent::FakeAgentFeature>()?;
    app.install::<ActivityFeature>()?;
    Ok(store)
}

fn settings_store(config: StoreConfig) -> StoreConfig {
    let config = config
        .or_in_memory()
        .configure(|store| store.rules(|rules| rules.on_unreadable(OnUnreadable::UseDefault)));
    #[cfg(debug_assertions)]
    let config = config.configure(|store| store.rules(|rules| rules.on_undeclared(guinea_plugin_store::amethystate::store::OnUndeclared::Drop)));
    config
}

#[cfg(test)]
mod tests {
    use domain::features::agents::settings::AgentSettings;
    use guinea::app::Harness;
    use guinea_plugin_store::amethystate::store::builder::StoreBuilder;
    use guinea_plugin_store::{StoreAccess, StoreConfig, StorePlugin};

    use super::settings_store;

    #[test]
    fn application_declarations_are_valid() {
        guinea::app::validate_declarations();
    }

    #[guinea::test(iterations = 1)]
    fn a_key_no_setting_declares_any_more_is_dropped_in_a_debug_build(h: &mut Harness) {
        let dir = tempfile::tempdir().unwrap();
        let at = dir.path().join("settings");
        let (earlier, _) = StoreBuilder::new(&at).migrate().unwrap();
        earlier.kv().namespace("agents").set("scan_interval_ms", &500u64).unwrap();
        AgentSettings::new_with(&earlier).ping_interval_ms().set(1000);
        earlier.close().unwrap();
        drop(earlier);

        let StorePlugin(store, _) = h.app(|app| super::install_store(app, settings_store(StoreConfig::at(&at)))).unwrap();

        let kept: Vec<String> = store
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

        let StorePlugin(store, _) = h.app(|app| super::install_store(app, settings_store(StoreConfig::at(&at)))).unwrap();

        let opened = store.try_settings::<AgentSettings>().map(|settings| settings.ping_interval_ms().get());
        assert!(matches!(opened, Ok(2000)), "{opened:?}");
    }

    #[guinea::test(iterations = 1)]
    fn a_store_file_that_will_not_open_leaves_the_settings_in_memory(h: &mut Harness) {
        let dir = tempfile::tempdir().unwrap();
        let at = dir.path().join("settings");
        std::fs::write(at.with_extension("redb"), "{ this never finished").unwrap();

        let installed = h.app(|app| super::install_store(app, settings_store(StoreConfig::at(&at))));

        assert!(installed.is_ok(), "{:?}", installed.as_ref().err());
        let StorePlugin(store, persistence) = installed.unwrap();
        let in_memory = persistence.map(|persistence| persistence.is_in_memory());
        assert_eq!(in_memory, Some(true));
        assert_eq!(store.try_settings::<AgentSettings>().expect("open test settings").ping_interval_ms().get(), 2000);
    }
}
