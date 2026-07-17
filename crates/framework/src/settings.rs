use crate::feature::{AppFeature, AppFeatureInitContext};
use amethystate::{DefaultStore, StoreBuilder, amethystate};
use anyhow::bail;
use std::path::PathBuf;
use tracing::debug;

#[amethystate(prefix = "settings.persistence")]
pub struct SettingsPersistenceSettings {
    #[amestate(default = 300u64)]
    pub save_debounce_ms: u64,

    #[amestate(default = 500u64)]
    pub watch_interval_ms: u64,
}

#[derive(Clone, Debug)]
pub struct SettingsPathOptions {
    pub windows_dir_name: String,
    pub unix_dir_name: String,
    pub file_name: String,
}

impl SettingsPathOptions {
    pub fn new(app_name: impl Into<String>) -> Self {
        let app_name = app_name.into();
        Self {
            windows_dir_name: app_name.clone(),
            unix_dir_name: app_name.to_ascii_lowercase(),
            file_name: "settings.json".to_string(),
        }
    }
}

impl Default for SettingsPathOptions {
    fn default() -> Self {
        Self {
            windows_dir_name: "Uniproc".to_string(),
            unix_dir_name: "uniproc".to_string(),
            file_name: "settings.json".to_string(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct SettingsFeature {
    path: Option<PathBuf>,
    path_options: SettingsPathOptions,
}

impl SettingsFeature {
    pub fn with_path(path: PathBuf) -> Self {
        Self {
            path: Some(path),
            path_options: SettingsPathOptions::default(),
        }
    }

    pub fn with_app_name(app_name: impl Into<String>) -> Self {
        Self {
            path: None,
            path_options: SettingsPathOptions::new(app_name),
        }
    }

    pub fn with_path_options(path_options: SettingsPathOptions) -> Self {
        Self {
            path: None,
            path_options,
        }
    }

    pub fn path(&self) -> anyhow::Result<PathBuf> {
        self.path
            .clone()
            .map(Ok)
            .unwrap_or_else(|| default_settings_path_with(&self.path_options))
    }
}

impl AppFeature for SettingsFeature {
    fn install(&mut self, ctx: &mut AppFeatureInitContext) -> anyhow::Result<()> {
        let store: DefaultStore = StoreBuilder::new(self.path()?).build()?;
        ctx.shared.insert(store);
        Ok(())
    }
}

pub fn default_settings_path() -> anyhow::Result<PathBuf> {
    default_settings_path_with(&SettingsPathOptions::default())
}

pub fn default_settings_path_with(options: &SettingsPathOptions) -> anyhow::Result<PathBuf> {
    if cfg!(target_os = "windows") {
        if let Ok(base) = std::env::var("APPDATA") {
            let path = PathBuf::from(base)
                .join(&options.windows_dir_name)
                .join(&options.file_name);
            debug!(path = %path.display(), "resolved default settings path (Windows/APPDATA)");
            return Ok(path);
        }
    } else {
        if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
            let path = PathBuf::from(xdg)
                .join(&options.unix_dir_name)
                .join(&options.file_name);
            debug!(path = %path.display(), "resolved default settings path (XDG_CONFIG_HOME)");
            return Ok(path);
        }

        if let Ok(home) = std::env::var("HOME") {
            let path = PathBuf::from(home)
                .join(".config")
                .join(&options.unix_dir_name)
                .join(&options.file_name);
            debug!(path = %path.display(), "resolved default settings path (~/.config)");
            return Ok(path);
        }
    }

    bail!(
        "Failed to resolve default settings directory. Please ensure APPDATA (Windows), XDG_CONFIG_HOME or HOME (Linux/macOS) environment variables are set."
    )
}
