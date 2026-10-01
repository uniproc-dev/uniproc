use app_contracts::features::settings::SidebarChart;

#[derive(guinea::Mark, Clone, Copy, PartialEq, Eq, Debug)]
pub enum SettingsMark {
    Theme,
    StartPage,
    UpdateSpeed,
    UpdateSpeedValue,
    ByteUnits,
    NetworkUnits,
    SidebarCharts,
    SidebarCpu,
    SidebarMemory,
    SidebarDisk,
    SidebarNetwork,
    SidebarGpu,
}

impl SettingsMark {
    pub fn sidebar_chart(chart: SidebarChart) -> Self {
        match chart {
            SidebarChart::Cpu => Self::SidebarCpu,
            SidebarChart::Memory => Self::SidebarMemory,
            SidebarChart::Disk => Self::SidebarDisk,
            SidebarChart::Network => Self::SidebarNetwork,
            SidebarChart::Gpu => Self::SidebarGpu,
        }
    }
}
