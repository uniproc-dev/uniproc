use std::sync::Arc;

use guinea::prelude::Event;
use serde::Deserialize;

use super::connection::AgentConnectionState;
use crate::ids::ids;

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

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, PartialOrd, Ord, Deserialize)]
pub enum Architecture {
    #[default]
    Unknown,
    X86,
    X64,
    Arm,
    Arm64,
    Arm64X86Compatible,
    Arm64X64Compatible,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Deserialize)]
pub enum UacVirtualization {
    #[default]
    Unknown,
    NotAllowed,
    Disabled,
    Enabled,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, PartialOrd, Ord, Deserialize)]
pub enum Isolation {
    #[default]
    Unknown,
    None,
    AppContainer,
    Uwp,
    Silo,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Deserialize)]
pub enum DpiAwareness {
    #[default]
    Unknown,
    Unaware,
    System,
    PerMonitor,
    PerMonitorV2,
    UnawareGdiScaled,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Deserialize)]
pub enum StackProtection {
    #[default]
    Unknown,
    Off,
    Compatible,
    Strict,
    CompatibleAudit,
    StrictAudit,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Deserialize)]
pub enum ExtendedCfg {
    #[default]
    Unknown,
    Off,
    Audit,
    On,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Deserialize)]
pub enum IoPriority {
    #[default]
    Unknown,
    VeryLow,
    Low,
    Normal,
    High,
    Critical,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Deserialize)]
pub struct Mitigations {
    pub dep: Option<bool>,
    pub stack_protection: StackProtection,
    pub extended_cfg: ExtendedCfg,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Deserialize)]
#[serde(default)]
pub struct ProcessRunState {
    pub suspended: Option<bool>,
    pub efficiency_mode: Option<bool>,
    pub base_priority: Option<ProcessPriority>,
    pub power_throttling: Option<bool>,
    pub job_object_id: u32,
    pub io_priority: IoPriority,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Deserialize)]
pub enum GpuEngineKind {
    #[default]
    Other,
    ThreeD,
    VideoDecode,
    VideoEncode,
    VideoProcessing,
    SceneAssembly,
    Copy,
    Overlay,
    Crypto,
    VideoCodec,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Deserialize)]
pub struct GpuEngineId {
    pub adapter_luid: u64,
    pub ordinal: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct WindowsGpuEngine {
    pub ordinal: u32,
    pub kind: GpuEngineKind,
    pub name: Arc<str>,
    pub busy_percent: f32,
    pub frequency_hz: u64,
    pub max_frequency_hz: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct WindowsGpu {
    pub luid: u64,
    pub name: Arc<str>,
    pub dedicated_limit_bytes: u64,
    pub dedicated_usage_bytes: u64,
    pub shared_limit_bytes: u64,
    pub shared_usage_bytes: u64,
    pub temperature_celsius: Option<f32>,
    pub fan_rpm: u32,
    pub power_percent: f32,
    pub memory_frequency_hz: u64,
    pub engines: Arc<[WindowsGpuEngine]>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct WindowsProcessorStats {
    pub busy_percent: f32,
    pub user_percent: f32,
    pub kernel_percent: f32,
    pub interrupt_percent: f32,
    pub dpc_percent: f32,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct WindowsMachineStats {
    pub total_physical_bytes: u64,
    pub available_physical_bytes: u64,
    pub commit_limit_bytes: u64,
    pub committed_bytes: u64,
    pub cpu_percent: f32,
    pub cpu_user_percent: f32,
    pub cpu_kernel_percent: f32,
    pub cpu_interrupt_percent: f32,
    pub cpu_dpc_percent: f32,
    pub cpu_max_mhz: u64,
    pub cpu_current_mhz: u64,
    pub processors: Arc<[WindowsProcessorStats]>,

    pub disk_read_bytes: u64,
    pub disk_write_bytes: u64,
    pub disk_read_iops: u64,
    pub disk_write_iops: u64,

    pub net_rx_bytes: u64,
    pub net_tx_bytes: u64,

    pub gpus: Arc<[WindowsGpu]>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct WindowsProcessStats {
    pub pid: u32,
    pub parent_pid: u32,
    pub session_id: u32,
    pub name: Arc<str>,
    pub first_arg: Arc<str>,
    pub command_line: Arc<str>,
    pub package_full_name: Arc<str>,
    pub package_relative_app_id: Arc<str>,
    pub cpu_percent: f32,
    pub cpu_cycles: u64,
    pub working_set_bytes: u64,
    pub commit_bytes: u64,
    pub peak_working_set_bytes: u64,
    pub private_working_set_bytes: u64,
    pub peak_commit_bytes: u64,
    pub virtual_size_bytes: u64,
    pub peak_virtual_size_bytes: u64,
    pub paged_pool_bytes: u64,
    pub peak_paged_pool_bytes: u64,
    pub non_paged_pool_bytes: u64,
    pub peak_non_paged_pool_bytes: u64,
    pub page_faults: u32,
    pub hard_faults: u32,
    pub handles: u32,
    pub threads: u32,
    pub peak_threads: u32,
    pub context_switches: u64,
    pub user_objects: u32,
    pub gdi_objects: u32,

    pub io_read_ops: u64,
    pub io_write_ops: u64,
    pub io_other_ops: u64,
    pub io_read_bytes: u64,
    pub io_write_bytes: u64,
    pub io_other_bytes: u64,

    pub disk_read_bytes: u64,
    pub disk_write_bytes: u64,
    pub disk_read_iops: u64,
    pub disk_write_iops: u64,
    pub disk_flush_ops: u64,

    pub net_rx_bytes: u64,
    pub net_tx_bytes: u64,

    pub gpu_percent: f32,
    pub gpu_engine: Option<GpuEngineId>,
    pub gpu_dedicated_bytes: u64,
    pub gpu_shared_bytes: u64,

    pub is_service: bool,
    pub is_kernel_process: bool,
    pub is_windows_process: bool,
    pub signature: SignatureStatus,
    pub image_path: Arc<str>,
    pub display_name: Arc<str>,
    pub console_host_pid: u32,
    pub start_time: u64,
    pub user: Arc<str>,
    pub architecture: Architecture,
    pub elevated: Option<bool>,
    pub uac_virtualization: UacVirtualization,
    pub isolation: Isolation,
    pub dpi_awareness: DpiAwareness,
    pub mitigations: Option<Mitigations>,
    pub publisher: Arc<str>,

    pub state: ProcessRunState,
}

impl WindowsMachineStats {
    pub fn used_physical_bytes(&self) -> u64 {
        self.total_physical_bytes.saturating_sub(self.available_physical_bytes)
    }

    pub fn gpu_percent(&self) -> f32 {
        self.gpus
            .iter()
            .flat_map(|gpu| gpu.engines.iter())
            .map(|engine| engine.busy_percent)
            .fold(0.0, f32::max)
    }

    pub fn gpu_dedicated_used_bytes(&self) -> u64 {
        self.gpus.iter().map(|gpu| gpu.dedicated_usage_bytes).sum()
    }
}

impl WindowsProcessStats {
    pub fn memory_bytes(&self) -> u64 {
        if self.private_working_set_bytes > 0 {
            self.private_working_set_bytes
        } else {
            self.working_set_bytes
        }
    }
}

ids! {
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Deserialize)]
    pub enum WindowsServiceState {
        #[default]
        Unknown => "unknown",
        Stopped => "stopped",
        StartPending => "start-pending",
        StopPending => "stop-pending",
        Running => "running",
        ContinuePending => "continue-pending",
        PausePending => "pause-pending",
        Paused => "paused",
    }
}

impl WindowsServiceState {
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
pub struct WindowsMachineSample {
    pub machine: Arc<WindowsMachineStats>,
    pub clock_100ns: u64,
}

#[derive(Clone, Debug, Event, Deserialize, guinea::Remote)]
#[remote(event)]
pub struct WindowsAgentRuntimeEvent {
    pub state: AgentConnectionState,
    pub latency_ms: Option<i32>,
}

#[derive(Clone, Debug, Event, Deserialize, guinea::Remote)]
#[remote(event)]
pub struct WindowsAgentInProcess;

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

#[derive(Clone, Debug, guinea::Request)]
#[request(reply = ActionOutcome)]
pub struct WindowsActionRequest(pub WindowsAction);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionOutcome {
    Done,
    Denied,
    Gone,
    Busy,
    NotConnected,
    Failed(u32),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ActionFailure {
    pub action: WindowsAction,
    pub target: Arc<str>,
    pub outcome: ActionOutcome,
}

