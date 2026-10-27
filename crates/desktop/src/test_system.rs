use std::path::PathBuf;
use std::sync::Mutex;

use domain::features::system::tools::Launch;
use domain::features::system::{SystemDeps, SystemFeature};
use guinea::app::Harness;
use domain::features::system::settings::SystemSettings;
use guinea_plugin_store::StoreAccess;

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

pub fn stored(store: &amethystate::Store) -> SystemSettings {
    store.try_settings::<SystemSettings>().expect("open test settings")
}

pub fn start(h: &mut Harness) -> amethystate::Store {
    LAUNCHED.lock().unwrap().clear();
    let store = crate::test_support::install(h);
    h.install::<SystemFeature>(&deps()).unwrap();
    store
}
