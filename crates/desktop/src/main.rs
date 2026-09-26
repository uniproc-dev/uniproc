#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod layouts;
mod pages;
mod route_memory;
mod routes;
#[cfg(test)]
mod test_agent;
mod tracing_init;
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
        .plugin(guinea_plugin_store::StorePlugin::for_app(
            meta::APP_NAME,
            "settings",
        ))
        .plugin(guinea_plugin_l10n::L10nPlugin::<app_contracts::l10n::L10n>::new("en"));

    #[cfg(debug_assertions)]
    let app = if std::env::var_os("UNIPROC_NO_DEVTOOLS").is_some() {
        app
    } else {
        app.plugin(guinea_plugin_devtools::DevToolsPlugin::new())
    };

    let app = app.feature(domain::features::agents::AgentsFeature);

    run(
        app,
        Window::new().title(meta::WINDOW_TITLE),
        route_memory::restore,
    )
}
