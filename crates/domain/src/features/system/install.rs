use std::path::PathBuf;

use app_contracts::features::system::SystemState;
use guinea::prelude::*;

use super::actor::{favourites, Rescan, SystemActor};
use super::settings::SystemSettings;
use super::tools::{self, Launch};

#[derive(Clone, Copy, Debug)]
pub struct SystemDeps {
    pub locate: fn(&str) -> Option<PathBuf>,
    pub launch: fn(Launch),
}

impl Default for SystemDeps {
    fn default() -> Self {
        Self {
            locate: tools::locate,
            launch: tools::launch,
        }
    }
}

feature! {
    pub SystemFeature {
        exports { SystemState }
    }
}

#[installs]
fn system(cx: &FeatureInitContext, deps: &SystemDeps) -> anyhow::Result<SystemFeature> {
    let deps = *deps;
    let settings = SystemSettings::new()?;
    let (pinned, frequent) = favourites(&settings);
    let seed = SystemState {
        pinned,
        frequent,
        ..SystemState::default()
    };
    let (system, addr) = cx
        .state::<SystemState>()
        .seed(seed)
        .driven_by(|push| SystemActor::new(push, deps, settings));
    addr.send(Rescan);
    Ok(SystemFeature(system))
}
