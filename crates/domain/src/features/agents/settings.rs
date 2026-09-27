use amethystate::amethystate;

#[amethystate(prefix = "agents")]
pub struct AgentSettings {
    #[amestate(default = 3u64)]
    pub connect_attempt_secs: u64,

    #[amestate(default = 2000u64)]
    pub ping_interval_ms: u64,

    #[amestate(default = 90u64)]
    pub wsl_connect_timeout_secs: u64,

    #[amestate(default = "Ubuntu".to_string())]
    pub wsl_distro: String,

    #[amestate(default = "/usr/local/bin/uniproc-agent".to_string())]
    pub wsl_agent_path: String,
}
