use amethystate::amethystate;

#[amethystate(prefix = "trace")]
pub struct TraceSettings {
    #[amestate(default = [])]
    pub enable_scopes: Vec<String>,
    #[amestate(default = [])]
    pub disable_scopes: Vec<String>,
    #[amestate(default = [])]
    pub disable_messages: Vec<String>,
    #[amestate(default = [])]
    pub disable_targets: Vec<String>,

    #[amestate(default = 64u64)]
    pub dump_capacity: u64,
}
