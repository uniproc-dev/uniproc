use std::sync::Arc;

use crate::features::agents::{WindowsServiceStartType, WindowsServiceState};
use crate::ids::ids;

ids! {
    #[derive(guinea::Mark, Clone, Copy, PartialEq, Eq, Debug, Hash, serde::Deserialize)]
    pub enum ServiceColumn {
        Name => "name",
        Status => "status",
        Pid => "pid",
        Group => "group",
        Description => "description",
        ServiceName => "service_name",
        ImagePath => "image_path",
        StartupType => "startup_type",
        Account => "account",
        ServiceType => "service_type"
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ServiceColumnConfig {
    pub width: u64,
    pub visible: bool,
}

impl ServiceColumn {
    pub fn default_config(self) -> ServiceColumnConfig {
        let width = match self {
            Self::Name => 260,
            Self::Status => 90,
            Self::Pid => 70,
            Self::Group => 120,
            Self::Description => 320,
            Self::ServiceName => 180,
            Self::ImagePath => 360,
            Self::StartupType => 160,
            Self::Account => 180,
            Self::ServiceType => 150,
        };
        ServiceColumnConfig {
            width,
            visible: matches!(
                self,
                Self::Name | Self::Status | Self::Pid | Self::Group | Self::Description
            ),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct ServiceRow {
    pub name: Arc<str>,
    pub display_name: Arc<str>,
    pub pid: u32,
    pub state: WindowsServiceState,
    pub group: Arc<str>,
    pub description: Arc<str>,
    pub image_path: Arc<str>,
    pub startup_type: WindowsServiceStartType,
    pub delayed_auto: bool,
    pub account: Arc<str>,
    pub service_type: u32,
    pub accepted_controls: u32,
}

impl ServiceRow {
    pub fn is_running(&self) -> bool {
        self.state.is_running()
    }

    pub fn can_start(&self) -> bool {
        self.state == WindowsServiceState::Stopped
            && self.startup_type != WindowsServiceStartType::Disabled
    }

    pub fn can_stop(&self) -> bool {
        matches!(
            self.state,
            WindowsServiceState::Running | WindowsServiceState::Paused
        ) && (self.service_type == 0 || self.accepted_controls & 1 != 0)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Deserialize)]
pub enum ServiceActionKind {
    Start,
    Stop,
    Pause,
    Resume,
    Restart,
}
