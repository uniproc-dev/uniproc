use std::ops::RangeInclusive;

use amethystate::amethystate;
use amethystate::store::{CheckContext, Invalid};

#[amethystate(prefix = "agents")]
pub struct AgentSettings {
    #[amestate(default = 3u64, check = Limits::connect_attempt)]
    pub connect_attempt_secs: u64,

    #[amestate(default = 2000u64, check = Limits::ping_interval)]
    pub ping_interval_ms: u64,

    #[amestate(default = 90u64, check = Limits::wsl_connect_timeout)]
    pub wsl_connect_timeout_secs: u64,

    #[amestate(default = "Ubuntu".to_string())]
    pub wsl_distro: String,

    #[amestate(default = "/usr/local/bin/uniproc-agent".to_string())]
    pub wsl_agent_path: String,
}

struct Limits;

#[expect(non_upper_case_globals)]
impl Limits {
    const ConnectAttemptSecs: RangeInclusive<u64> = 1..=60;
    const PingIntervalMs: RangeInclusive<u64> = 250..=60_000;
    const WslConnectTimeoutSecs: RangeInclusive<u64> = 1..=600;

    fn connect_attempt(secs: &mut u64, _: &CheckContext) -> Result<(), Invalid> {
        Self::fit(secs, Self::ConnectAttemptSecs)
    }

    fn ping_interval(ms: &mut u64, _: &CheckContext) -> Result<(), Invalid> {
        Self::fit(ms, Self::PingIntervalMs)
    }

    fn wsl_connect_timeout(secs: &mut u64, _: &CheckContext) -> Result<(), Invalid> {
        Self::fit(secs, Self::WslConnectTimeoutSecs)
    }

    fn fit(value: &mut u64, range: RangeInclusive<u64>) -> Result<(), Invalid> {
        *value = (*value).clamp(*range.start(), *range.end());
        Ok(())
    }
}
