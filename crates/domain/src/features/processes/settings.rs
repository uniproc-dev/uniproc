use amethystate::{AmeType, ReactiveMap, amethystate};
use serde::{Deserialize, Serialize};

#[amethystate(prefix = "process")]
pub struct ProcessSettings {
    #[amestate(default = 1500u64)]
    scan_interval_ms: u64,

    #[amestate(default = 5000u64)]
    terminate_timeout_ms: u64,

    #[amestate(nested)]
    columns: ColumnsSettings,
}

#[amethystate]
pub struct ColumnsSettings {
    #[amestate(default = 70u64)]
    default_width_px: u64,

    #[amestate(default = {
        "name": 200u64,
        "cpu": 90u64,
        "memory": 120u64,
    })]
    widths_px: ReactiveMap<String, u64>,

    #[amestate(default = {
        "name": ColumnMetadata { is_text: true, ..Default::default() },
        "cpu": ColumnMetadata { is_metric: true, ..Default::default() },
        "memory": ColumnMetadata { is_metric: true, ..Default::default() },
    })]
    column_metadata: ReactiveMap<String, ColumnMetadata>,

    #[amestate(default = {
        "name": 120u64,
        "cpu": 90u64,
        "memory": 120u64,
    })]
    min_widths_px: ReactiveMap<String, u64>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, AmeType)]
pub struct ColumnMetadata {
    #[serde(default)]
    pub is_text: bool,

    #[serde(default)]
    pub is_metric: bool,
}
