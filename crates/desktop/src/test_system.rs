use std::path::PathBuf;
use std::sync::Mutex;

use domain::features::system::tools::Launch;
use domain::features::system::{SystemDeps, SystemFeature};
use guinea::app::Harness;
use guinea_plugin_l10n::L10nPlugin;
use guinea_plugin_store::amethystate::store::builder::Backend;
use guinea_plugin_store::StorePlugin;

static LAUNCHED: Mutex<Vec<Launch>> = Mutex::new(Vec::new());

pub fn launched() -> Vec<Launch> {
    LAUNCHED.lock().unwrap().clone()
}

fn record(launch: Launch) {
    LAUNCHED.lock().unwrap().push(launch);
}

fn procexp_only(name: &str) -> Option<PathBuf> {
    (name == "procexp64.exe").then(|| PathBuf::from(r"C:\Tools\procexp64.exe"))
}

pub fn launch(file: &str, args: &str) -> Launch {
    Launch {
        file: file.into(),
        args: args.into(),
    }
}

pub fn deps() -> SystemDeps {
    SystemDeps {
        locate: procexp_only,
        launch: record,
    }
}

pub fn start(h: &mut Harness) -> tempfile::TempDir {
    LAUNCHED.lock().unwrap().clear();
    let dir = tempfile::tempdir().unwrap();
    h.plugin(StorePlugin::at(dir.path().join("settings")).backend(Backend::Json))
        .unwrap()
        .plugin(L10nPlugin::<app_contracts::l10n::L10n>::new("en"))
        .unwrap();
    h.install::<SystemFeature>(&deps()).unwrap();
    dir
}
