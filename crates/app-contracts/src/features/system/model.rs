#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ToolGroup {
    Windows,
    Settings,
    Sysinternals,
}

impl ToolGroup {
    pub const ALL: [Self; 3] = [Self::Windows, Self::Settings, Self::Sysinternals];
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, serde::Deserialize)]
pub enum SystemTool {
    TaskManager,
    ResourceMonitor,
    PerformanceMonitor,
    ReliabilityMonitor,
    SystemInformation,
    DirectXDiagnostic,
    EventViewer,
    Services,
    TaskScheduler,
    DeviceManager,
    DiskManagement,
    ComputerManagement,
    RegistryEditor,
    SystemProperties,
    EnvironmentVariables,
    StartupApps,
    InstalledApps,
    Storage,
    Power,
    WindowsUpdate,
    About,
    ProcessExplorer,
    ProcessMonitor,
    Autoruns,
    TcpView,
    RamMap,
    VmMap,
}

impl SystemTool {
    pub const ALL: [Self; 27] = [
        Self::TaskManager,
        Self::ResourceMonitor,
        Self::PerformanceMonitor,
        Self::ReliabilityMonitor,
        Self::SystemInformation,
        Self::DirectXDiagnostic,
        Self::EventViewer,
        Self::Services,
        Self::TaskScheduler,
        Self::DeviceManager,
        Self::DiskManagement,
        Self::ComputerManagement,
        Self::RegistryEditor,
        Self::SystemProperties,
        Self::EnvironmentVariables,
        Self::StartupApps,
        Self::InstalledApps,
        Self::Storage,
        Self::Power,
        Self::WindowsUpdate,
        Self::About,
        Self::ProcessExplorer,
        Self::ProcessMonitor,
        Self::Autoruns,
        Self::TcpView,
        Self::RamMap,
        Self::VmMap,
    ];

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
}
