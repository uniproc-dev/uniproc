use amethystate::{ReactiveMap, amethystate};
use app_contracts::features::processes::{ColumnConfig, PinnedProcess};

#[amethystate(prefix = "processes")]
pub struct ProcessesSettings {
    #[amestate(default = 500u64)]
    scan_interval_ms: u64,

    #[amestate(nested)]
    columns: ProcessesColumnsSettings,

    #[amestate(nested)]
    grouping: ProcessesGroupingSettings,
}

#[amethystate]
pub struct ProcessesGroupingSettings {
    #[amestate(default = {})]
    collapsed_sections: ReactiveMap<String, bool>,

    #[amestate(default = {})]
    pins: ReactiveMap<String, PinnedProcess>,

    #[amestate(default = true)]
    by_type: bool,
}

#[amethystate]
pub struct ProcessesColumnsSettings {
    #[amestate(default = {
        "name": ColumnConfig { width: 280, visible: true },
        "cpu": ColumnConfig { width: 120, visible: true },
        "memory": ColumnConfig { width: 140, visible: true },
        "net": ColumnConfig { width: 110, visible: true },
        "disk": ColumnConfig { width: 110, visible: true },
    })]
    configs: ReactiveMap<String, ColumnConfig>,
}
