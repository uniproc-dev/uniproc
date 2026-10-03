#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod layouts;
mod pages;
mod route_memory;
mod routes;
#[cfg(test)]
mod test_agent;
#[cfg(test)]
mod test_page;
#[cfg(test)]
mod test_system;
mod tracing_init;
mod window_press;
mod xaml_resources;

mod meta {
    guinea::meta::manifest!();
}

use guinea::app::GuineaApp;
use guinea::winui::{Window, run};

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn main() -> anyhow::Result<()> {
    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::builder()
        .file_name(std::env::var_os("UNIPROC_DHAT_FILE").unwrap_or_else(|| "dhat-heap.json".into()))
        .build();

    tracing_init::init()?;

    let app = GuineaApp::new()
        .meta(guinea::app_meta!())
        .plugin(guinea_plugin_single_instance::SingleInstancePlugin::new())
        .plugin(settings_store(guinea_plugin_store::StorePlugin::for_app(
            meta::APP_NAME,
            "settings",
        )))
        .plugin(guinea_plugin_l10n::L10nPlugin::<app_contracts::l10n::L10n>::new("en"));

    #[cfg(debug_assertions)]
    let app = if std::env::var_os("UNIPROC_NO_DEVTOOLS").is_some() {
        app
    } else {
        app.plugin(guinea_plugin_devtools::DevToolsPlugin::new().launch(true))
    };

    let app = app.feature(domain::features::agents::AgentsFeature);

    run(
        app,
        Window::new().title(meta::WINDOW_TITLE),
        route_memory::restore,
    )
}

fn settings_store(plugin: guinea_plugin_store::StorePlugin) -> guinea_plugin_store::StorePlugin {
    #[cfg(debug_assertions)]
    let plugin = plugin.configure(|store| {
        store.rules(|rules| rules.on_undeclared(guinea_plugin_store::amethystate::store::OnUndeclared::Drop))
    });
    plugin
}

#[cfg(test)]
mod tests {
    use domain::features::agents::settings::AgentSettings;
    use guinea::app::Harness;
    use guinea_plugin_store::amethystate::store::builder::StoreBuilder;
    use guinea_plugin_store::{StoreAccess, StorePlugin};

    use super::settings_store;

    #[guinea::test(iterations = 1, exclusive = "store")]
    fn a_key_no_setting_declares_any_more_is_dropped_in_a_debug_build(h: &mut Harness) {
        let dir = tempfile::tempdir().unwrap();
        let at = dir.path().join("settings");
        let (earlier, _) = StoreBuilder::new(&at).migrate().unwrap();
        earlier.kv().namespace("agents").set("scan_interval_ms", &500u64).unwrap();
        AgentSettings::new_with(&earlier).unwrap().ping_interval_ms().set(1000).unwrap();
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
}
