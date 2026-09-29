use std::time::Duration;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, serde::Deserialize)]
pub enum AppTheme {
    #[default]
    System,
    Light,
    Dark,
}

impl AppTheme {
    pub const ALL: [AppTheme; 3] = [AppTheme::System, AppTheme::Light, AppTheme::Dark];

    pub fn id(self) -> &'static str {
        match self {
            AppTheme::System => "system",
            AppTheme::Light => "light",
            AppTheme::Dark => "dark",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|theme| theme.id() == id)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, serde::Deserialize)]
pub enum StartPage {
    #[default]
    LastOpened,
    Processes,
    Services,
    Wsl,
}

impl StartPage {
    pub const ALL: [StartPage; 4] = [
        StartPage::LastOpened,
        StartPage::Processes,
        StartPage::Services,
        StartPage::Wsl,
    ];

    pub fn id(self) -> &'static str {
        match self {
            StartPage::LastOpened => "last_opened",
            StartPage::Processes => "processes",
            StartPage::Services => "services",
            StartPage::Wsl => "wsl",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|page| page.id() == id)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, serde::Deserialize)]
pub enum ByteUnits {
    #[default]
    Windows,
    Iec,
}

impl ByteUnits {
    pub const ALL: [ByteUnits; 2] = [ByteUnits::Windows, ByteUnits::Iec];

    pub fn id(self) -> &'static str {
        match self {
            ByteUnits::Windows => "windows",
            ByteUnits::Iec => "iec",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|units| units.id() == id)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Deserialize)]
pub enum SidebarChart {
    Cpu,
    Memory,
    Disk,
    Network,
    Gpu,
}

impl SidebarChart {
    pub const ALL: [SidebarChart; 5] = [
        SidebarChart::Cpu,
        SidebarChart::Memory,
        SidebarChart::Disk,
        SidebarChart::Network,
        SidebarChart::Gpu,
    ];

    pub fn id(self) -> &'static str {
        match self {
            SidebarChart::Cpu => "cpu",
            SidebarChart::Memory => "memory",
            SidebarChart::Disk => "disk",
            SidebarChart::Network => "network",
            SidebarChart::Gpu => "gpu",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|chart| chart.id() == id)
    }

    fn bit(self) -> u8 {
        1 << self as u8
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct SidebarCharts {
    hidden: u8,
}

impl SidebarCharts {
    pub fn shows(self, chart: SidebarChart) -> bool {
        self.hidden & chart.bit() == 0
    }

    pub fn with(self, chart: SidebarChart, shown: bool) -> Self {
        let hidden = if shown {
            self.hidden & !chart.bit()
        } else {
            self.hidden | chart.bit()
        };
        Self { hidden }
    }

    pub fn shown(self) -> impl Iterator<Item = SidebarChart> {
        SidebarChart::ALL.into_iter().filter(move |chart| self.shows(*chart))
    }

    pub fn from_hidden_ids(ids: &str) -> Self {
        ids.split(',')
            .filter_map(SidebarChart::from_id)
            .fold(Self::default(), |charts, chart| charts.with(chart, false))
    }

    pub fn hidden_ids(self) -> String {
        SidebarChart::ALL
            .into_iter()
            .filter(|chart| !self.shows(*chart))
            .map(SidebarChart::id)
            .collect::<Vec<_>>()
            .join(",")
    }
}

pub struct UpdateInterval;

#[expect(non_upper_case_globals)]
impl UpdateInterval {
    pub const Min: Duration = Duration::from_millis(100);
    pub const Max: Duration = Duration::from_secs(5);
    pub const Step: Duration = Duration::from_millis(100);
    pub const Default: Duration = Duration::from_millis(1500);
    pub const Machine: Duration = Duration::from_millis(200);

    pub fn clamp(ms: u64) -> Duration {
        let step = Self::Step.as_millis() as u64;
        let rounded = (ms + step / 2) / step * step;
        Duration::from_millis(rounded).clamp(Self::Min, Self::Max)
    }}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_round_trip() {
        for theme in AppTheme::ALL {
            assert_eq!(AppTheme::from_id(theme.id()), Some(theme));
        }
        for page in StartPage::ALL {
            assert_eq!(StartPage::from_id(page.id()), Some(page));
        }
        for units in ByteUnits::ALL {
            assert_eq!(ByteUnits::from_id(units.id()), Some(units));
        }
        for chart in SidebarChart::ALL {
            assert_eq!(SidebarChart::from_id(chart.id()), Some(chart));
        }
        assert_eq!(AppTheme::from_id("sepia"), None);
    }

    #[test]
    fn sidebar_charts_keep_what_was_hidden() {
        let charts = SidebarCharts::default()
            .with(SidebarChart::Disk, false)
            .with(SidebarChart::Gpu, false);
        assert_eq!(charts.hidden_ids(), "disk,gpu");
        assert_eq!(SidebarCharts::from_hidden_ids("disk,gpu,sepia"), charts);
        assert_eq!(
            charts.shown().collect::<Vec<_>>(),
            [SidebarChart::Cpu, SidebarChart::Memory, SidebarChart::Network]
        );
        assert!(charts.with(SidebarChart::Disk, true).shows(SidebarChart::Disk));
        assert_eq!(SidebarCharts::from_hidden_ids(""), SidebarCharts::default());
    }

    #[test]
    fn an_interval_lands_on_a_step_inside_the_range() {
        assert_eq!(UpdateInterval::clamp(0), UpdateInterval::Min);
        assert_eq!(UpdateInterval::clamp(60_000), UpdateInterval::Max);
        assert_eq!(UpdateInterval::clamp(1_449), Duration::from_millis(1_400));
        assert_eq!(UpdateInterval::clamp(1_450), Duration::from_millis(1_500));
    }}
