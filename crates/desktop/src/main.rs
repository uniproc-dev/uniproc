#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod layouts;
mod pages;
mod parts;
mod route_memory;
mod routes;
#[cfg(test)]
mod test_agent;
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
    domain::features::agent_link::wait_for_the_copy_it_replaces();

    run(
        GuineaApp::new().meta(guinea::app_meta!()).application::<app::App>(),
        Window::new().title(meta::WINDOW_TITLE),
        route_memory::restore,
    )
}
