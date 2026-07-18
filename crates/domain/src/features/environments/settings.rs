use amethystate::amethystate;

#[amethystate(prefix = "environments")]
pub struct EnvironmentsSettings {
    #[amestate(default = 5000u64)]
    scan_interval_ms: u64,
}
