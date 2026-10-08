use std::fs::File;
use std::io;
use std::path::PathBuf;
use std::sync::Mutex;

use tracing_subscriber::Layer;
use tracing_subscriber::filter::{LevelFilter, Targets};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

const LOG_FILE: &str = "run_desktop.log";

const OWN_CRATES: [&str; 5] = ["uniproc", "domain", "ui", "app_contracts", "context"];

fn own(rest: LevelFilter) -> Targets {
    OWN_CRATES
        .into_iter()
        .fold(Targets::new().with_default(rest), |targets, krate| targets.with_target(krate, LevelFilter::DEBUG))
}

fn log_path() -> Option<PathBuf> {
    if cfg!(debug_assertions) {
        return Some(PathBuf::from(LOG_FILE));
    }
    let dir = PathBuf::from(std::env::var_os("LOCALAPPDATA")?)
        .join(crate::meta::APP_NAME)
        .join("logs");
    std::fs::create_dir_all(&dir).ok()?;
    Some(dir.join(LOG_FILE))
}

pub fn init() -> anyhow::Result<()> {
    let observed = Targets::new()
        .with_default(LevelFilter::DEBUG)
        .with_target("ogurpchik", LevelFilter::WARN);

    let file = log_path()
        .and_then(|path| File::create(path).ok())
        .map(|file| guinea::core::trace::json(Mutex::new(file)).with_filter(own(LevelFilter::WARN)));

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_writer(io::stderr).with_filter(own(LevelFilter::OFF)))
        .with(file)
        .with(guinea::core::observability::layer().with_filter(observed))
        .try_init()?;
    Ok(())
}
