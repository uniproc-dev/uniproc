use amethystate::{ReactiveMap, amethystate};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ToolUse {
    pub count: u32,
    pub last_ms: u64,
}

#[amethystate(prefix = "system")]
pub struct SystemSettings {
    #[amestate(default = {})]
    uses: ReactiveMap<String, ToolUse>,

    #[amestate(default = {})]
    pinned: ReactiveMap<String, u64>,
}
