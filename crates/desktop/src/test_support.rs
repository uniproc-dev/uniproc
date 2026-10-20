use amethystate::Store;
use app_contracts::l10n::L10n;
use guinea::app::Harness;
use guinea::winui::harness::Mounted;
use guinea::winui::Window;
use guinea_plugin_l10n::{L10nConfig, L10nPlugin};
use guinea_plugin_store::{StoreConfig, StorePlugin};

pub fn install(h: &mut Harness) -> &mut Harness {
    h.app(|app| {
        let StorePlugin(store, _) = app.plugin::<StorePlugin>(StoreConfig::in_memory())?;
        app.plugin::<L10nPlugin<L10n>>(L10nConfig::new("en").remembered_in(store.clone()))?;
        app.provide(store);
        Ok(())
    }).unwrap();
    h
}

pub fn store(h: &Harness) -> Store {
    h.application_scope().service::<Store>().expect("test settings store was not provided")
}

pub fn mount_application(h: &Harness, route: crate::routes::Route) -> anyhow::Result<Mounted<'_, crate::routes::Route>> {
    Mounted::from_window(h, Window::of::<crate::layouts::MainWindow>().at(route))
}
