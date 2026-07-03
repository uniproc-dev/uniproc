use rpstate::rpstate;

#[rpstate(prefix = "settings.persistence")]
pub struct SettingsPersistenceSettings {
    #[state(default = 300u64)]
    pub save_debounce_ms: u64,

    #[state(default = 500u64)]
    pub watch_interval_ms: u64,
}
