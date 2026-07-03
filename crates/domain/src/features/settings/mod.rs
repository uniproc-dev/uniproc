mod settings;

use crate::features::settings::settings::SettingsPersistenceSettings;
use anyhow::bail;
use framework::feature::{AppFeature, AppFeatureInitContext};
use macros::app_feature;
use rpstate::{DefaultStore, StoreBuilder};
use std::path::PathBuf;
use std::sync::Arc;
use tracing::debug;

#[app_feature]
pub fn settings_feature(
    ctx: &mut AppFeatureInitContext,
    params: Option<PathBuf>,
) -> anyhow::Result<()> {
    let path = params
        .clone()
        .map(Ok)
        .unwrap_or_else(default_settings_path)?;

    let store = StoreBuilder::new(path).build()?;

    ctx.shared.insert_arc(store.clone());

    Ok(())
}

impl Default for SettingsFeature {
    fn default() -> Self {
        Self::new(None)
    }
}

impl SettingsFeature {
    pub fn with_path(path: PathBuf) -> Self {
        Self::new(Some(path))
    }
}

pub fn default_settings_path() -> anyhow::Result<PathBuf> {
    if cfg!(target_os = "windows") {
        if let Ok(base) = std::env::var("APPDATA") {
            let p = PathBuf::from(base).join("Uniproc").join("settings.json");
            debug!(path = %p.display(), "resolved default settings path (Windows/APPDATA)");
            return Ok(p);
        }
    } else {
        if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
            let p = PathBuf::from(xdg).join("uniproc").join("settings.json");
            debug!(path = %p.display(), "resolved default settings path (XDG_CONFIG_HOME)");
            return Ok(p);
        }

        if let Ok(home) = std::env::var("HOME") {
            let p = PathBuf::from(home)
                .join(".config")
                .join("uniproc")
                .join("settings.json");
            debug!(path = %p.display(), "resolved default settings path (~/.config)");
            return Ok(p);
        }
    }

    bail!(
        "Failed to resolve default settings directory. Please ensure APPDATA (Windows), XDG_CONFIG_HOME or HOME (Linux/macOS) environment variables are set."
    )
}
