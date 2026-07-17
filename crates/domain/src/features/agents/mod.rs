pub mod actor;
pub mod backend;
pub mod connection;
pub mod providers;
pub mod settings;

use crate::agents_impl::providers::{windows, wsl};
use forsl::feature::{AppFeature, AppFeatureDeinitContext, AppFeatureInitContext};
use macros::app_feature;
use tracing::info;

#[app_feature]
pub fn agents_feature(ctx: &mut AppFeatureInitContext) -> anyhow::Result<()> {
    info!("Agents feature installed");
    cfg_if::cfg_if! {
        if #[cfg(target_os = "windows")] {
            wsl::wsl_agent_feature(ctx)?;
            windows::windows_agent_feature(ctx)?;
        } else {
            linux::linux_agent_feature(ctx)?;
        }
    }

    Ok(())
}
