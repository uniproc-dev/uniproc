use std::sync::Arc;

use guinea::prelude::Event;
use serde::Deserialize;
use uuid::Uuid;

use super::connection::AgentConnectionState;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Deserialize)]
pub enum SignatureStatus {
    #[default]
    Unknown,
    Unsigned,
    Microsoft,
    ThirdParty,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize)]
pub enum ProcessPriority {
    Idle,
    BelowNormal,
    Normal,
    AboveNormal,
    High,
    Realtime,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct WindowsMachineStats {
    pub total_physical_kb: u64,
    pub available_physical_kb: u64,
    pub used_physical_kb: u64,
    pub cpu_percent: f32,
    pub cpu_max_mhz: u64,
    pub cpu_current_mhz: u64,

    pub disk_read_bytes: u64,
    pub disk_write_bytes: u64,
    pub disk_read_iops: u64,
    pub disk_write_iops: u64,

    pub net_rx_bytes: u64,
    pub net_tx_bytes: u64,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct WindowsProcessStats {
    pub pid: u32,
    pub parent_pid: u32,
    pub session_id: u32,
    pub name: Arc<str>,
    pub first_arg: Arc<str>,
    pub package_full_name: Arc<str>,
    pub package_relative_app_id: Arc<str>,
    pub cpu_percent: f32,
    pub working_set_kb: u64,
    pub private_bytes_kb: u64,
    pub peak_working_set_kb: u64,
    pub private_working_set_kb: u64,

    pub disk_read_bytes: u64,
    pub disk_write_bytes: u64,
    pub disk_read_iops: u64,
    pub disk_write_iops: u64,

    pub net_rx_bytes: u64,
    pub net_tx_bytes: u64,

    pub is_service: bool,
    pub is_kernel_process: bool,
    pub is_windows_process: bool,
    pub signature: SignatureStatus,
    pub image_path: Arc<str>,
    pub display_name: Arc<str>,
    pub console_host_pid: u32,
}

impl WindowsProcessStats {
    pub fn memory_kb(&self) -> u64 {
        if self.private_working_set_kb > 0 {
            self.private_working_set_kb
        } else {
            self.working_set_kb
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Deserialize)]
pub enum WindowsServiceState {
    #[default]
    Unknown,
    Stopped,
    StartPending,
    StopPending,
    Running,
    ContinuePending,
    PausePending,
    Paused,
}

impl WindowsServiceState {
    pub fn id(self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::Stopped => "stopped",
            Self::StartPending => "start-pending",
            Self::StopPending => "stop-pending",
            Self::Running => "running",
            Self::ContinuePending => "continue-pending",
            Self::PausePending => "pause-pending",
            Self::Paused => "paused",
        }
    }

    pub fn is_running(self) -> bool {
        matches!(self, Self::Running)
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct WindowsServiceStats {
    pub name: Arc<str>,
    pub display_name: Arc<str>,
    pub pid: u32,
    pub state: WindowsServiceState,
    pub load_group: Arc<str>,
    pub description: Arc<str>,
    pub image_path: Arc<str>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct WindowsReport {
    pub machine: WindowsMachineStats,
    pub processes: Vec<WindowsProcessStats>,
    pub services: Vec<WindowsServiceStats>,
}

#[derive(Clone, Debug, Event, Deserialize, guinea::Remote)]
#[remote(event)]
pub enum WindowsReportMessage {
    Report(Arc<WindowsReport>),
    Unavailable(AgentConnectionState),
}

#[derive(Clone, Debug, Event, Deserialize, guinea::Remote)]
#[remote(event)]
pub struct WindowsAgentRuntimeEvent {
    pub state: AgentConnectionState,
    pub latency_ms: Option<i32>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub enum WindowsAction {
    Kill { pid: u32 },
    Suspend { pid: u32 },
    Resume { pid: u32 },
    SetPriority { pid: u32, priority: ProcessPriority },
    SetAffinity { pid: u32, mask: u64 },
    ServiceStart { name: String },
    ServiceStop { name: String },
    ServicePause { name: String },
    ServiceResume { name: String },
    ServiceRestart { name: String },
}

#[derive(Clone, Debug, Event, Deserialize, guinea::Remote)]
#[remote(event)]
pub struct WindowsActionRequest {
    pub correlation_id: Uuid,
    pub action: WindowsAction,
}

impl WindowsActionRequest {
    pub fn new(correlation_id: Uuid, action: WindowsAction) -> Self {
        Self { correlation_id, action }
    }
}

#[derive(Clone, Debug, Event, Deserialize, guinea::Remote)]
#[remote(event)]
pub struct WindowsActionResponse {
    pub correlation_id: Uuid,
    pub code: u32,
}

impl WindowsActionResponse {
    pub fn new(correlation_id: Uuid, code: u32) -> Self {
        Self { correlation_id, code }
    }

    pub fn succeeded(&self) -> bool {
        self.code == 0
    }
}

