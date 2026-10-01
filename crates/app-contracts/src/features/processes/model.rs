use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::features::agents::{Architecture, EnvironmentKind, Isolation, ProcessPriority};
use crate::ids::ids;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColumnConfig {
    pub width: u64,
    pub visible: bool,
}

impl Default for ColumnConfig {
    fn default() -> Self {
        Self {
            width: 110,
            visible: true,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PinnedProcess {
    pub exe_path: String,
    pub package_full_name: String,
    pub display_name: String,
}

ids! {
    #[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
    pub enum ProcessCategory {
        App => "app",
        BackgroundMicrosoft => "background-microsoft",
        BackgroundThirdParty => "background-third-party",
        Wsl => "wsl",
        WindowsService => "windows-service",
        WindowsKernel => "windows-kernel",
    }
}

impl ProcessCategory {
    pub fn classify(
        has_visible_window: bool,
        is_kernel_process: bool,
        is_service: bool,
        is_microsoft_signed: bool,
    ) -> Self {
        if has_visible_window {
            Self::App
        } else if is_kernel_process {
            Self::WindowsKernel
        } else if is_service {
            Self::WindowsService
        } else if is_microsoft_signed {
            Self::BackgroundMicrosoft
        } else {
            Self::BackgroundThirdParty
        }
    }

    pub const ORDER: [Self; 6] = [
        Self::App,
        Self::BackgroundThirdParty,
        Self::Wsl,
        Self::BackgroundMicrosoft,
        Self::WindowsService,
        Self::WindowsKernel,
    ];

    pub fn takes_actions(self) -> bool {
        !matches!(self, Self::WindowsKernel | Self::Wsl)
    }
}

ids! {
    #[derive(guinea::Mark, Clone, Copy, PartialEq, Eq, Debug, Hash, serde::Deserialize)]
    pub enum ProcessColumn {
        Name => "name",
        Pid => "pid",
        ProcessName => "process_name",
        Cpu => "cpu",
        Memory => "memory",
        Net => "net",
        Disk => "disk",
        Gpu => "gpu",
        GpuMemory => "gpu_memory",
        Status => "status",
        Publisher => "publisher",
        User => "user",
        CommandLine => "command_line",
        ImagePath => "image_path",
        GpuEngine => "gpu_engine",
        Platform => "platform",
        Elevated => "elevated",
        Isolation => "isolation",
    }
}

impl ProcessColumn {
    pub fn default_config(self) -> ColumnConfig {
        let (width, visible) = match self {
            Self::Name => (280, true),
            Self::Pid => (80, false),
            Self::ProcessName => (160, false),
            Self::Cpu => (120, true),
            Self::Memory => (140, true),
            Self::Net | Self::Disk => (110, true),
            Self::Gpu => (100, true),
            Self::GpuMemory => (130, false),
            Self::Status => (120, false),
            Self::Publisher => (180, false),
            Self::User => (140, false),
            Self::CommandLine => (320, false),
            Self::ImagePath => (280, false),
            Self::GpuEngine => (120, false),
            Self::Platform => (90, false),
            Self::Elevated => (80, false),
            Self::Isolation => (110, false),
        };
        ColumnConfig { width, visible }
    }

    pub fn is_metric(self) -> bool {
        matches!(
            self,
            Self::Cpu | Self::Memory | Self::Net | Self::Disk | Self::Gpu | Self::GpuMemory
        )
    }

    pub fn sorts_ascending_first(self) -> bool {
        !self.is_metric()
    }

    pub fn from_mark(name: &str) -> Option<Self> {
        use guinea::Mark;
        Self::ALL.into_iter().find(|c| c.name() == name)
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct HostedService {
    pub name: Arc<str>,
    pub display_name: Arc<str>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ProcessWindow {
    pub handle: isize,
    pub title: Arc<str>,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
pub enum ProcessStatus {
    #[default]
    Running,
    Suspended,
    Efficiency,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct GpuEngineLabel {
    pub adapter: u32,
    pub engine: Arc<str>,
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct ProcessDetails {
    pub status: ProcessStatus,
    pub priority: Option<ProcessPriority>,
    pub publisher: Arc<str>,
    pub user: Arc<str>,
    pub command_line: Arc<str>,
    pub gpu_engine: Option<GpuEngineLabel>,
    pub architecture: Architecture,
    pub elevated: Option<bool>,
    pub isolation: Isolation,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Owner {
    pub name: Arc<str>,
    pub others: usize,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ProcessRow {
    pub pid: u32,
    pub name: Arc<str>,
    pub display_name: Arc<str>,
    pub cpu_percent: f32,
    pub memory_bytes: u64,
    pub disk_bytes: u64,
    pub net_bytes: u64,
    pub gpu_percent: f32,
    pub gpu_memory_bytes: u64,
    pub exe_path: Arc<str>,
    pub package_full_name: Arc<str>,
    pub owner: Option<Owner>,
    pub owner_pid: Option<u32>,
    pub category: ProcessCategory,
    pub services: Option<Arc<[HostedService]>>,
    pub windows: Option<Arc<[ProcessWindow]>>,
    pub details: Arc<ProcessDetails>,
    pub is_monitor: bool,
}

impl ProcessRow {
    pub fn takes_actions(&self) -> bool {
        self.category.takes_actions() && !self.is_monitor
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct WslProcess {
    pub global_pid: u32,
    pub row: ProcessRow,
}

#[derive(Clone, PartialEq, Debug)]
pub struct WslEnvironment {
    pub pid_ns: u64,
    pub name: Arc<str>,
    pub kind: EnvironmentKind,
    pub processes: Arc<[WslProcess]>,
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct MachineSummary {
    pub cpu_percent: f32,
    pub cpu_current_mhz: u64,
    pub cpu_max_mhz: u64,
    pub memory_used_bytes: u64,
    pub memory_total_bytes: u64,
    pub gpu_percent: f32,
    pub gpu_memory_used_bytes: u64,
    pub disk_bytes_per_sec: u64,
    pub network_bytes_per_sec: u64,
}
