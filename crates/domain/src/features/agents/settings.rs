use rpstate::rpstate;

#[rpstate(prefix = "agents")]
pub struct AgentSettings {
    #[state(default = 8u64)]
    pub connect_timeout_secs: u64,

    #[state(default = 2000u64)]
    pub ping_interval_ms: u64,
}
