use rpstate::rpstate;

#[rpstate(prefix = "trace")]
pub struct TraceSettings {
    #[state(default = [])]
    pub enable_scopes: Vec<String>,
    #[state(default = [])]
    pub disable_scopes: Vec<String>,
    #[state(default = [])]
    pub disable_messages: Vec<String>,
    #[state(default = [])]
    pub disable_targets: Vec<String>,

    #[state(default = 64u64)]
    pub dump_capacity: u64,
}
