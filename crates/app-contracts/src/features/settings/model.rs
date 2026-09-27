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

pub struct UpdateInterval;

#[expect(non_upper_case_globals)]
impl UpdateInterval {
    pub const Min: Duration = Duration::from_millis(100);
    pub const Max: Duration = Duration::from_secs(5);
    pub const Step: Duration = Duration::from_millis(100);
    pub const Default: Duration = Duration::from_millis(1500);

    pub fn clamp(ms: u64) -> Duration {
        let step = Self::Step.as_millis() as u64;
        let rounded = (ms + step / 2) / step * step;
        Duration::from_millis(rounded).clamp(Self::Min, Self::Max)
    }
}

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
        assert_eq!(AppTheme::from_id("sepia"), None);
    }

    #[test]
    fn an_interval_lands_on_a_step_inside_the_range() {
        assert_eq!(UpdateInterval::clamp(0), UpdateInterval::Min);
        assert_eq!(UpdateInterval::clamp(60_000), UpdateInterval::Max);
        assert_eq!(UpdateInterval::clamp(1_449), Duration::from_millis(1_400));
        assert_eq!(UpdateInterval::clamp(1_450), Duration::from_millis(1_500));
    }
}
