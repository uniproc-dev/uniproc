use std::sync::Arc;

use serde::{Deserialize, Serialize};

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

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum ProcessCategory {
    App,
    BackgroundMicrosoft,
    BackgroundThirdParty,
    WindowsService,
    WindowsKernel,
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

    pub const ORDER: [Self; 5] = [
        Self::App,
        Self::BackgroundThirdParty,
        Self::BackgroundMicrosoft,
        Self::WindowsService,
        Self::WindowsKernel,
    ];

    pub fn id(self) -> &'static str {
        match self {
            Self::App => "app",
            Self::BackgroundThirdParty => "background-third-party",
            Self::BackgroundMicrosoft => "background-microsoft",
            Self::WindowsService => "windows-service",
            Self::WindowsKernel => "windows-kernel",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ORDER.into_iter().find(|c| c.id() == id)
    }

    pub fn takes_actions(self) -> bool {
        self != Self::WindowsKernel
    }
}

#[derive(guinea::Mark, Clone, Copy, PartialEq, Eq, Debug, Hash, serde::Deserialize)]
pub enum ProcessColumn {
    Name,
    Cpu,
    Memory,
    Net,
    Disk,
}

impl ProcessColumn {
    pub const ALL: [Self; 5] = [Self::Name, Self::Cpu, Self::Memory, Self::Net, Self::Disk];

    pub fn id(self) -> &'static str {
        match self {
            Self::Name => "name",
            Self::Cpu => "cpu",
            Self::Memory => "memory",
            Self::Net => "net",
            Self::Disk => "disk",
        }
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

#[derive(Clone, PartialEq, Debug)]
pub struct ProcessRow {
    pub pid: u32,
    pub name: Arc<str>,
    pub display_name: Arc<str>,
    pub cpu_percent: f32,
    pub memory_bytes: u64,
    pub disk_bytes: u64,
    pub net_bytes: u64,
    pub exe_path: Arc<str>,
    pub package_full_name: Arc<str>,
    pub owner: Option<Arc<str>>,
    pub owner_pid: Option<u32>,
    pub category: ProcessCategory,
    pub services: Option<Arc<[HostedService]>>,
    pub windows: Option<Arc<[ProcessWindow]>>,
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct MachineSummary {
    pub cpu_percent: f32,
    pub cpu_current_mhz: u64,
    pub cpu_max_mhz: u64,
    pub memory_used_bytes: u64,
    pub memory_total_bytes: u64,
}
