use amethystate::Store;
use app_contracts::l10n::L10n;
use guinea::app::Harness;
use guinea::winui::harness::Mounted;
use guinea::winui::Window;
use guinea_plugin_l10n::{L10nConfig, L10nPlugin};
use guinea_plugin_store::{StoreConfig, StorePlugin};
use crate::platform::{Platform, PlatformDeps};

pub fn install(h: &mut Harness) -> Store {
    install_with(h, PlatformDeps::default())
}

pub fn install_with(h: &mut Harness, deps: PlatformDeps) -> Store {
    h.app(|app| {
        app.plugin::<Platform>(deps)?;
        let StorePlugin(store, _) = app.plugin::<StorePlugin>(StoreConfig::in_memory())?;
        app.plugin::<L10nPlugin<L10n>>(L10nConfig::new("en").remembered_in(store.clone()))?;
        Ok(store)
    }).unwrap()
}

pub fn mount_application(h: &Harness, route: crate::routes::Route) -> anyhow::Result<Mounted<'_, crate::routes::Route>> {
    Mounted::from_window(h, Window::of::<crate::layouts::MainWindow>().at(route))
}
