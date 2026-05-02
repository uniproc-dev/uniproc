use macros::feature_settings;

#[feature_settings(prefix = "environments")]
pub struct EnvironmentsSettings {
    #[setting(default = 5000u64)]
    scan_interval_ms: u64,
}
