use rpstate::rpstate;

#[rpstate(prefix = "environments")]
pub struct EnvironmentsSettings {
    #[state(default = 5000u64)]
    scan_interval_ms: u64,
}
