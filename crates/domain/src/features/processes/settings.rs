use rpstate::{rpstate, ReactiveMap, RpType};
use serde::{Deserialize, Serialize};

#[rpstate(prefix = "process")]
pub struct ProcessSettings {
    #[state(default = 1500u64)]
    scan_interval_ms: u64,

    #[state(default = 5000u64)]
    terminate_timeout_ms: u64,

    #[state(nested)]
    columns: ColumnsSettings,
}

#[rpstate]
pub struct ColumnsSettings {
    #[state(default = 70u64)]
    default_width_px: u64,

    #[state(default = {
        "name": 200u64,
        "cpu": 90u64,
        "memory": 120u64,
    })]
    widths_px: ReactiveMap<String, u64>,

    #[setting(default = {
        "name": ColumnMetadata { is_text: true, ..Default::default() },
        "cpu": ColumnMetadata { is_metric: true, ..Default::default() },
        "memory": ColumnMetadata { is_metric: true, ..Default::default() },
    })]
    column_metadata: ReactiveMap<String, ColumnMetadata>,

    #[setting(default = {
        "name": 120u64,
        "cpu": 90u64,
        "memory": 120u64,
    })]
    min_widths_px: ReactiveMap<String, u64>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, RpType)]
pub struct ColumnMetadata {
    #[serde(default)]
    pub is_text: bool,

    #[serde(default)]
    pub is_metric: bool,
}
