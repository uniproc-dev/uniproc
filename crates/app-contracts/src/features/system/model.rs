use crate::ids::ids;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ToolGroup {
    Windows,
    Settings,
    Sysinternals,
}

impl ToolGroup {
    pub const ALL: [Self; 3] = [Self::Windows, Self::Settings, Self::Sysinternals];
}

ids! {
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, serde::Deserialize)]
    pub enum SystemTool {
        TaskManager => "task_manager",
        ResourceMonitor => "resource_monitor",
        PerformanceMonitor => "performance_monitor",
        ReliabilityMonitor => "reliability_monitor",
        SystemInformation => "system_information",
        DirectXDiagnostic => "directx_diagnostic",
        EventViewer => "event_viewer",
        Services => "services",
        TaskScheduler => "task_scheduler",
        DeviceManager => "device_manager",
        DiskManagement => "disk_management",
        ComputerManagement => "computer_management",
        RegistryEditor => "registry_editor",
        SystemProperties => "system_properties",
        EnvironmentVariables => "environment_variables",
        StartupApps => "startup_apps",
        InstalledApps => "installed_apps",
        Storage => "storage",
        Power => "power",
        WindowsUpdate => "windows_update",
        About => "about",
        ProcessExplorer => "process_explorer",
        ProcessMonitor => "process_monitor",
        Autoruns => "autoruns",
        TcpView => "tcpview",
        RamMap => "rammap",
        VmMap => "vmmap",
    }
}

impl SystemTool {
    pub fn group(self) -> ToolGroup {
        match self {
            Self::StartupApps
            | Self::InstalledApps
            | Self::Storage
            | Self::Power
            | Self::WindowsUpdate
            | Self::About => ToolGroup::Settings,
            Self::ProcessExplorer
            | Self::ProcessMonitor
            | Self::Autoruns
            | Self::TcpView
            | Self::RamMap
            | Self::VmMap => ToolGroup::Sysinternals,
            _ => ToolGroup::Windows,
        }
    }

    pub fn in_group(group: ToolGroup) -> impl Iterator<Item = Self> {
        Self::ALL.into_iter().filter(move |tool| tool.group() == group)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_tool_is_listed_once() {
        let mut seen = std::collections::HashSet::new();
        for tool in SystemTool::ALL {
            assert!(seen.insert(tool), "{tool:?} is listed twice");
        }
        let grouped: usize = ToolGroup::ALL.iter().map(|group| SystemTool::in_group(*group).count()).sum();
        assert_eq!(grouped, SystemTool::ALL.len());
    }

    #[test]
    fn ids_come_back_as_the_same_tool() {
        for tool in SystemTool::ALL {
            assert_eq!(SystemTool::from_id(tool.id()), Some(tool));
        }
        assert_eq!(SystemTool::from_id("gone"), None);
    }
}
