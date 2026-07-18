use amethystate::{AmeType, ReactiveMap, amethystate};
use serde::{Deserialize, Serialize};

#[amethystate(prefix = "services")]
pub struct ServiceSettings {
    #[amestate(default = 2000u64)]
    scan_interval_ms: u64,

    #[amestate(nested)]
    columns: ServiceColumnsSettings,
}

#[amethystate]
pub struct ServiceColumnsSettings {
    #[amestate(default = 70u64)]
    default_width_px: u64,

    #[amestate(default = {
        "name": 150u64,
        "pid": 80u64,
        "status": 100u64,
        "group": 120u64,
        "description": 100u64,
    })]
    widths_px: ReactiveMap<String, u64>,

    #[amestate(default = {
        "name": 20u64,
        "pid": 20u64,
        "status": 20u64,
        "group": 20u64,
        "description": 20u64,
    })]
    min_widths_px: ReactiveMap<String, u64>,

    #[amestate(default = {
        "display_name": ServiceColumnMetadata { is_text: true },
        "status": ServiceColumnMetadata { is_text: true },
        "description": ServiceColumnMetadata { is_text: true },
        "pid": ServiceColumnMetadata { is_text: true },
        "group": ServiceColumnMetadata { is_text: true },
    })]
    column_metadata: ReactiveMap<String, ServiceColumnMetadata>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, AmeType)]
pub struct ServiceColumnMetadata {
    #[serde(default)]
    pub is_text: bool,
}
