use std::fs::File;
use std::io;
use std::path::PathBuf;
use std::sync::Mutex;

use tracing_subscriber::filter::{LevelFilter, Targets};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

const LOG_FILE: &str = "run_desktop.log";

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
    let targets = Targets::new()
        .with_default(LevelFilter::DEBUG)
        .with_target("ogurpchik", LevelFilter::WARN);

    let file = log_path()
        .and_then(|path| File::create(path).ok())
        .map(|file| guinea::core::trace::json(Mutex::new(file)));

    tracing_subscriber::registry()
        .with(targets)
        .with(tracing_subscriber::fmt::layer().with_writer(io::stderr))
        .with(file)
        .with(guinea::core::devtools::layer())
        .try_init()?;
    Ok(())
}
