use std::fs::File;
use std::io;
use std::sync::Mutex;

use tracing_subscriber::filter::{LevelFilter, Targets};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

const LOG_FILE: &str = "run_desktop.log";

pub fn init() -> anyhow::Result<()> {
    let targets = Targets::new()
        .with_default(LevelFilter::DEBUG)
        .with_target("ogurpchik", LevelFilter::WARN);

    let file = File::create(LOG_FILE)
        .ok()
        .map(|file| guinea::core::trace::json(Mutex::new(file)));

    tracing_subscriber::registry()
        .with(targets)
        .with(tracing_subscriber::fmt::layer().with_writer(io::stderr))
        .with(file)
        .with(guinea::core::devtools::layer())
        .try_init()?;
    Ok(())
}
